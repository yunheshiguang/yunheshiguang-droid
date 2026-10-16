use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_contract::yunheshiguang::documents::v1::DocumentsChanged;
use yunheshiguang_sdk::documents::DocumentStore;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::automation::FLOWS_COLLECTION;
use crate::workspace::batch::uses_host_files;
use crate::workspace::blocking::run_blocking;
use crate::workspace::clock;
use crate::workspace::contract::*;
use crate::workspace::documents::DOCUMENTS_CHANGED_EVENT;
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::scripts::SCRIPTS_COLLECTION;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::templates::TemplateCatalog;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::format::{
    Loaded, check_ids, check_manifest, read_package, read_package_text, storage_failed,
    write_package,
};
use super::library::{collect, install, plan, preview, read_library};
use super::signing::SigningKeys;

pub struct PackagesService {
    pub(super) documents: Arc<dyn DocumentStore>,
    pub(super) catalog: Arc<TemplateCatalog>,
    pub(super) governance: Arc<Governance>,
    pub(super) telemetry: Arc<Telemetry>,
    pub(super) events: Arc<dyn WorkspaceEvents>,
    pub(super) keys: Arc<SigningKeys>,
}

fn carries_host_files(package: &TemplatePackage) -> bool {
    package
        .flows
        .iter()
        .filter_map(|entry| entry.flow.as_option())
        .flat_map(|flow| flow.steps.iter())
        .any(|step| uses_host_files(step.kind.to_i32()))
}

fn count(items: usize) -> u32 {
    u32::try_from(items).unwrap_or(u32::MAX)
}

fn trimmed(manifest: PackageManifest) -> PackageManifest {
    PackageManifest {
        id: manifest.id.trim().to_string(),
        name: manifest.name.trim().to_string(),
        version: manifest.version.trim().to_string(),
        author: manifest.author.trim().to_string(),
        description: manifest.description.trim().to_string(),
        created_at: manifest.created_at,
        ..Default::default()
    }
}

impl PackagesService {
    pub fn market(&self) -> MarketPackages {
        let granted = self.governance.granted_capabilities();

        MarketPackages {
            packages: self
                .catalog
                .packages(&granted)
                .into_iter()
                .map(|(info, package)| MarketPackage {
                    source_id: info.id,
                    source: info.label,
                    flows: count(package.flows.len()),
                    scripts: count(package.scripts.len()),
                    manifest: package.manifest,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn load(&self, origin: Option<PackageOrigin>) -> Result<Loaded, UiText> {
        let mut loaded = match origin.and_then(|origin| origin.origin) {
            Some(package_origin::Origin::Path(path)) => {
                let path = PathBuf::from(path.trim());
                if !path.is_absolute() {
                    return Err(ui_text(PACKAGES_PATH_INVALID_KEY));
                }
                read_package(&path)?
            }
            Some(package_origin::Origin::Json(text)) => read_package_text(&text)?,
            Some(package_origin::Origin::Market(reference)) => self
                .catalog
                .packages(&self.governance.granted_capabilities())
                .into_iter()
                .find(|(info, package)| {
                    info.id == reference.source_id
                        && package
                            .manifest
                            .as_option()
                            .is_some_and(|manifest| manifest.id == reference.package_id)
                })
                .map(|(_, package)| Loaded {
                    package,
                    verified: true,
                    signer: None,
                })
                .ok_or_else(|| ui_text(PACKAGES_MARKET_MISSING_KEY))?,
            None => return Err(ui_text(PACKAGES_ORIGIN_REQUIRED_KEY)),
        };

        if let Some(signed) = &loaded.signer {
            loaded.verified = self.keys.is_trusted(&signed.public_key);
        }
        check_manifest(
            &loaded
                .package
                .manifest
                .clone()
                .into_option()
                .unwrap_or_default(),
        )?;
        check_ids(loaded.package.flows.iter().map(|item| item.id.as_str()))?;
        check_ids(loaded.package.scripts.iter().map(|item| item.id.as_str()))?;
        Ok(loaded)
    }

    fn record(&self, started: Instant, succeeded: bool) {
        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(GovernedAction::ManagePackages, succeeded, started.elapsed())
                .with_targets(1, u32::from(!succeeded)),
        ));
    }

    pub async fn inspect(self: &Arc<Self>, request: InspectPackageRequest) -> InspectPackageResult {
        let service = Arc::clone(self);
        let outcome = run_blocking(move || {
            Ok(service.load(request.origin.into()).and_then(|loaded| {
                let library = read_library(service.documents.as_ref())?;
                Ok(PackagePreview {
                    signer: (loaded.signer.map(|signed| PackageSigner {
                        trusted: service.keys.is_trusted(&signed.public_key),
                        key_id: signed.key_id,
                        name: service
                            .keys
                            .trusted_name(&signed.public_key)
                            .unwrap_or(signed.name),
                        public_key: signed.public_key,
                        ..Default::default()
                    }))
                    .into(),
                    ..preview(&loaded.package, &library, loaded.verified)
                })
            }))
        })
        .await
        .unwrap_or_else(|error| Err(storage_failed(error)));

        match outcome {
            Ok(preview) => InspectPackageResult {
                accepted: true,
                detail: Default::default(),
                preview: preview.into(),
                ..Default::default()
            },
            Err(detail) => InspectPackageResult {
                accepted: false,
                detail: detail.into(),
                preview: Default::default(),
                ..Default::default()
            },
        }
    }

    pub async fn install(
        self: &Arc<Self>,
        request: InstallPackageRequest,
        principal: &Principal,
    ) -> InstallPackageResult {
        let refused = |detail: UiText| InstallPackageResult {
            accepted: false,
            detail: detail.into(),
            ..InstallPackageResult::default()
        };
        let service = Arc::clone(self);
        let origin = request.origin;
        let loaded = match run_blocking(move || Ok(service.load(origin.into())))
            .await
            .unwrap_or_else(|error| Err(storage_failed(error)))
        {
            Ok(loaded) => loaded,
            Err(detail) => return refused(detail),
        };
        let manifest = loaded
            .package
            .manifest
            .clone()
            .into_option()
            .unwrap_or_default();
        let ticket = match self
            .governance
            .begin(GovernedAction::ManagePackages, principal, &[])
        {
            Ok(ticket) => ticket,
            Err(reason) => return refused(reason),
        };
        if principal.is_api() && carries_host_files(&loaded.package) {
            let reason = ui_text(API_HOST_FILES_KEY);
            ticket.finish(AuditOutcome::Denied, Some(reason.clone()));
            return refused(reason);
        }
        let started = Instant::now();
        let policy = ConflictPolicy::known(request.on_conflict).unwrap_or_default();
        let documents = Arc::clone(&self.documents);
        let outcome = run_blocking(move || {
            Ok(read_library(documents.as_ref()).and_then(|library| {
                let plan = plan(&loaded.package, &library, policy);
                install(documents.as_ref(), &plan)?;
                Ok((count(plan.flows.len()), count(plan.scripts.len())))
            }))
        })
        .await
        .unwrap_or_else(|error| Err(storage_failed(error)));
        self.record(started, outcome.is_ok());

        match outcome {
            Ok((flows, scripts)) => {
                ticket.finish(
                    AuditOutcome::Succeeded,
                    Some(ui_text_with_params(
                        PACKAGES_AUDIT_INSTALLED_KEY,
                        "",
                        [
                            ("name", manifest.name.clone()),
                            ("version", manifest.version.clone()),
                        ],
                    )),
                );
                for collection in [SCRIPTS_COLLECTION, FLOWS_COLLECTION] {
                    notify(
                        self.events.as_ref(),
                        DOCUMENTS_CHANGED_EVENT,
                        &DocumentsChanged {
                            collection: collection.to_string(),
                            ..Default::default()
                        },
                    );
                }
                InstallPackageResult {
                    accepted: true,
                    detail: ui_text_with_params(
                        PACKAGES_INSTALLED_KEY,
                        "",
                        [
                            ("flows", flows.to_string()),
                            ("scripts", scripts.to_string()),
                        ],
                    )
                    .into(),
                    flows,
                    scripts,
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                refused(detail)
            }
        }
    }

    pub async fn trust_signer(
        self: &Arc<Self>,
        request: TrustPackageSignerRequest,
        principal: &Principal,
    ) -> TrustPackageSignerResult {
        let refused = |detail: UiText| TrustPackageSignerResult {
            accepted: false,
            detail: detail.into(),
            ..Default::default()
        };
        let ticket = match self
            .governance
            .begin(GovernedAction::ManagePackages, principal, &[])
        {
            Ok(ticket) => ticket,
            Err(reason) => return refused(reason),
        };
        let started = Instant::now();
        let keys = Arc::clone(&self.keys);
        let name = request.name.trim().to_string();
        let trusted_name = name.clone();
        let outcome = run_blocking(move || Ok(keys.trust(&request.public_key, &trusted_name)))
            .await
            .unwrap_or_else(|error| Err(storage_failed(error)));
        self.record(started, outcome.is_ok());

        match outcome {
            Ok(()) => {
                let named = |key| ui_text_with_params(key, "", [("name", name.clone())]);
                ticket.finish(
                    AuditOutcome::Succeeded,
                    Some(named(PACKAGES_AUDIT_SIGNER_TRUSTED_KEY)),
                );
                TrustPackageSignerResult {
                    accepted: true,
                    detail: named(PACKAGES_SIGNER_TRUSTED_KEY).into(),
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                refused(detail)
            }
        }
    }

    pub async fn export(
        self: &Arc<Self>,
        request: ExportPackageRequest,
        principal: &Principal,
    ) -> ExportPackageResult {
        let refused = |detail: UiText| ExportPackageResult {
            accepted: false,
            detail: detail.into(),
            ..ExportPackageResult::default()
        };
        let manifest = trimmed(request.manifest.into_option().unwrap_or_default());
        let path = PathBuf::from(request.path.trim());

        if let Err(detail) = check_manifest(&manifest) {
            return refused(detail);
        }
        if request.flow_ids.is_empty() {
            return refused(ui_text(PACKAGES_NO_FLOWS_KEY));
        }
        if !path.is_absolute() {
            return refused(ui_text(PACKAGES_PATH_INVALID_KEY));
        }

        let ticket = match self
            .governance
            .begin(GovernedAction::ManagePackages, principal, &[])
        {
            Ok(ticket) => ticket,
            Err(reason) => return refused(reason),
        };
        let started = Instant::now();
        let documents = Arc::clone(&self.documents);
        let destination = path.clone();
        let packaged = PackageManifest {
            created_at: clock::now().into(),
            ..manifest.clone()
        };
        let flow_ids = request.flow_ids;
        let keys = Arc::clone(&self.keys);
        let signer = principal.operator.name.clone();
        let outcome = run_blocking(move || {
            Ok(read_library(documents.as_ref()).and_then(|library| {
                let (flows, scripts) = collect(&library, &flow_ids)?;
                let counts = (count(flows.len()), count(scripts.len()));
                let package = TemplatePackage {
                    manifest: packaged.into(),
                    flows,
                    scripts,
                    ..TemplatePackage::default()
                };
                write_package(&destination, &package, &keys, &signer)?;
                Ok(counts)
            }))
        })
        .await
        .unwrap_or_else(|error| Err(storage_failed(error)));
        self.record(started, outcome.is_ok());

        match outcome {
            Ok((flows, scripts)) => {
                ticket.finish(
                    AuditOutcome::Succeeded,
                    Some(ui_text_with_params(
                        PACKAGES_AUDIT_EXPORTED_KEY,
                        "",
                        [
                            ("name", manifest.name.clone()),
                            ("version", manifest.version.clone()),
                        ],
                    )),
                );
                ExportPackageResult {
                    accepted: true,
                    detail: ui_text_with_params(
                        PACKAGES_EXPORTED_KEY,
                        "",
                        [
                            ("flows", flows.to_string()),
                            ("scripts", scripts.to_string()),
                            ("path", path.display().to_string()),
                        ],
                    )
                    .into(),
                    flows,
                    scripts,
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                refused(detail)
            }
        }
    }
}
