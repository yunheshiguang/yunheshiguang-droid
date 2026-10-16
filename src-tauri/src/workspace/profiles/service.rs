use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::adb::{AdbService, DeviceIdentities};
use crate::workspace::android_tools::{AndroidPlatformToolsRegistry, probe_android_tools};
use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::{
    AuditOutcome, DeviceGroupDefinition, DeviceProfilesChanged, GovernedAction, OrganizeResult,
    RemoveDeviceGroupRequest, SaveDeviceGroupRequest, UiText, UpdateDeviceProfilesRequest,
};
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::ui_text_with_params;

use super::book::ProfileBook;
use super::keys::connected_device_keys;
use super::patch::{CheckedPatch, check_group};

pub const DEVICE_PROFILES_CHANGED_EVENT: &str = "workspace:device-profiles-changed";

struct Change {
    updated: u32,
    audit: UiText,
}

pub struct ProfilesService {
    pub(super) book: Arc<ProfileBook>,
    pub(super) identities: Arc<DeviceIdentities>,
    pub(super) registry: Arc<AndroidPlatformToolsRegistry>,
    pub(super) adb: Arc<AdbService>,
    pub(super) governance: Arc<Governance>,
    pub(super) telemetry: Arc<Telemetry>,
    pub(super) events: Arc<dyn WorkspaceEvents>,
}

fn storage_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        ORGANIZE_STORAGE_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn refused(detail: UiText) -> OrganizeResult {
    OrganizeResult {
        accepted: false,
        detail: detail.into(),
        updated: 0,
        ..Default::default()
    }
}

impl ProfilesService {
    fn device_keys(&self) -> HashMap<String, String> {
        let _runtime = self.adb.acquire_runtime();
        let Some(adb_path) = probe_android_tools(&self.registry.config()).adb_path else {
            return HashMap::new();
        };
        let environment = self.adb.probe_path_locked(&adb_path);

        connected_device_keys(&environment.devices, Some(&adb_path), &self.identities)
    }

    async fn organize(
        self: &Arc<Self>,
        principal: &Principal,
        targets: &[String],
        groups: &[String],
        work: impl FnOnce(&ProfilesService) -> Result<Change, UiText> + Send + 'static,
    ) -> OrganizeResult {
        let ticket = match self.governance.begin_in(
            GovernedAction::OrganizeDevices,
            principal,
            targets,
            groups,
        ) {
            Ok(ticket) => ticket,
            Err(reason) => {
                log::info!("profiles: denied by governance");
                return refused(reason);
            }
        };
        let service = Arc::clone(self);
        let started = Instant::now();
        let outcome = run_blocking(move || Ok(work(&service)))
            .await
            .unwrap_or_else(|error| Err(storage_failed(error)));

        let result = match outcome {
            Ok(change) => {
                ticket.finish(AuditOutcome::Succeeded, Some(change.audit));
                OrganizeResult {
                    accepted: true,
                    detail: Default::default(),
                    updated: change.updated,
                    ..Default::default()
                }
            }
            Err(detail) => {
                log::warn!("profiles: change failed: {}", detail.key);
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                refused(detail)
            }
        };

        let targets = u32::try_from(targets.len().max(1)).unwrap_or(u32::MAX);
        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(
                GovernedAction::OrganizeDevices,
                result.accepted,
                started.elapsed(),
            )
            .with_targets(targets, if result.accepted { 0 } else { targets }),
        ));

        if result.updated > 0 {
            notify(
                self.events.as_ref(),
                DEVICE_PROFILES_CHANGED_EVENT,
                &DeviceProfilesChanged {
                    ..Default::default()
                },
            );
        }

        result
    }

    pub async fn update_profiles(
        self: &Arc<Self>,
        request: UpdateDeviceProfilesRequest,
        principal: &Principal,
    ) -> OrganizeResult {
        let patch = match CheckedPatch::check(
            &request.patch.clone().into_option().unwrap_or_default(),
            request.device_ids.len(),
        ) {
            Ok(patch) => patch,
            Err(detail) => return refused(detail),
        };
        let device_ids = request.device_ids;
        let targets = device_ids.clone();
        let destination = patch.destination_group().into_iter().collect::<Vec<_>>();

        self.organize(principal, &targets, &destination, move |service| {
            let keys = service.device_keys();
            let inventory = service.book.inventory();
            patch.check_group(&inventory)?;

            let mut changed = Vec::new();
            let mut retired = Vec::new();

            for (index, device_id) in device_ids.iter().enumerate() {
                let key = keys
                    .get(device_id)
                    .cloned()
                    .unwrap_or_else(|| device_id.clone());
                let mut profile = inventory
                    .profile(&key, device_id)
                    .cloned()
                    .unwrap_or_default();

                if !profile.device_key.is_empty() && profile.device_key != key {
                    retired.push(std::mem::take(&mut profile.device_key));
                }
                profile.device_key = key;
                patch.apply(&mut profile, index)?;
                changed.push(profile);
            }

            service
                .book
                .save_profiles(&changed, &retired)
                .map_err(storage_failed)?;

            let updated = u32::try_from(changed.len()).unwrap_or(u32::MAX);
            Ok(Change {
                updated,
                audit: ui_text_with_params(
                    ORGANIZE_AUDIT_PROFILES_KEY,
                    "",
                    [("count", updated.to_string())],
                ),
            })
        })
        .await
    }

    pub async fn save_group(
        self: &Arc<Self>,
        request: SaveDeviceGroupRequest,
        principal: &Principal,
    ) -> OrganizeResult {
        let group: DeviceGroupDefinition = match check_group(request.group.into()) {
            Ok(group) => group,
            Err(detail) => return refused(detail),
        };

        let affected = [group.id.clone()];

        self.organize(principal, &[], &affected, move |service| {
            service.book.save_group(&group).map_err(storage_failed)?;

            Ok(Change {
                updated: 1,
                audit: ui_text_with_params(
                    ORGANIZE_AUDIT_GROUP_SAVED_KEY,
                    "",
                    [("name", group.name)],
                ),
            })
        })
        .await
    }

    pub async fn remove_group(
        self: &Arc<Self>,
        request: RemoveDeviceGroupRequest,
        principal: &Principal,
    ) -> OrganizeResult {
        let id = request.id.trim().to_string();
        let affected = [id.clone()];

        self.organize(principal, &[], &affected, move |service| {
            let name = service
                .book
                .inventory()
                .group(&id)
                .map(|group| group.name.clone())
                .unwrap_or_default();
            service.book.remove_group(&id).map_err(storage_failed)?;

            Ok(Change {
                updated: 1,
                audit: ui_text_with_params(ORGANIZE_AUDIT_GROUP_REMOVED_KEY, "", [("name", name)]),
            })
        })
        .await
    }
}
