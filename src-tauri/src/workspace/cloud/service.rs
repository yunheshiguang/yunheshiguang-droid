use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use yunheshiguang_sdk::cloud::CloudPhoneProvider;
use yunheshiguang_sdk::connection::Endpoint;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::*;
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::registered::{REGISTERED_PROVIDER_ID, RegisteredPhones, storage_failed};

pub const CLOUD_PHONES_CHANGED_EVENT: &str = "workspace:cloud-phones-changed";
const LIST_TIMEOUT: Duration = Duration::from_secs(10);
const POWER_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_NAME_CHARS: usize = 80;

pub struct CloudService {
    pub(super) providers: Vec<Arc<dyn CloudPhoneProvider>>,
    pub(super) registered: Arc<RegisteredPhones>,
    pub(super) governance: Arc<Governance>,
    pub(super) telemetry: Arc<Telemetry>,
    pub(super) events: Arc<dyn WorkspaceEvents>,
    pub(super) cache: Mutex<Vec<CloudPhone>>,
}

fn refused(detail: UiText) -> CloudPhoneResult {
    CloudPhoneResult {
        accepted: false,
        detail: detail.into(),
        phone: Default::default(),
        ..Default::default()
    }
}

fn target_of(phone: &CloudPhone) -> String {
    phone.endpoint.as_option().map_or_else(
        || phone.id.clone(),
        |endpoint| format!("{}:{}", endpoint.host, endpoint.port),
    )
}

#[derive(Clone, Copy)]
enum Power {
    Start,
    Stop,
}

impl CloudService {
    pub fn cached(&self) -> Vec<CloudPhone> {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn provider(&self, id: &str) -> Option<&Arc<dyn CloudPhoneProvider>> {
        self.providers
            .iter()
            .find(|provider| provider.describe().id == id)
    }

    pub async fn list(&self) -> CloudPhones {
        let mut listing = CloudPhones::default();

        for provider in &self.providers {
            let info = provider.describe();
            let phones = tokio::time::timeout(LIST_TIMEOUT, provider.phones())
                .await
                .unwrap_or_else(|_| Err(ui_text(CLOUD_TIMED_OUT_KEY)));

            match phones {
                Ok(phones) => listing
                    .phones
                    .extend(phones.into_iter().map(|phone| CloudPhone {
                        provider_id: info.id.clone(),
                        ..phone
                    })),
                Err(problem) => {
                    log::info!("cloud {}: listing failed: {}", info.id, problem.key);
                    listing.problems.push(ProviderProblem {
                        provider_id: info.id.clone(),
                        problem: problem.into(),
                        ..Default::default()
                    });
                }
            }
            listing.providers.push(info);
        }

        listing
            .phones
            .clone_into(&mut self.cache.lock().unwrap_or_else(PoisonError::into_inner));
        listing
    }

    fn record(&self, started: Instant, succeeded: bool) {
        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(
                GovernedAction::ManageCloudPhones,
                succeeded,
                started.elapsed(),
            )
            .with_targets(1, u32::from(!succeeded)),
        ));
    }

    async fn manage(
        &self,
        principal: &Principal,
        target: String,
        audit_key: &'static str,
        work: impl Future<Output = Result<CloudPhone, UiText>>,
    ) -> CloudPhoneResult {
        let ticket = match self.governance.begin(
            GovernedAction::ManageCloudPhones,
            principal,
            std::slice::from_ref(&target),
        ) {
            Ok(ticket) => ticket,
            Err(reason) => return refused(reason),
        };
        let started = Instant::now();
        let outcome = work.await;
        self.record(started, outcome.is_ok());

        match outcome {
            Ok(phone) => {
                ticket.finish(
                    AuditOutcome::Succeeded,
                    Some(ui_text_with_params(
                        audit_key,
                        "",
                        [("name", phone.name.clone())],
                    )),
                );
                self.list().await;
                notify(
                    self.events.as_ref(),
                    CLOUD_PHONES_CHANGED_EVENT,
                    &CloudPhonesChanged {
                        ..Default::default()
                    },
                );
                CloudPhoneResult {
                    accepted: true,
                    detail: Default::default(),
                    phone: phone.into(),
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                refused(detail)
            }
        }
    }

    async fn power(
        &self,
        request: CloudPhoneRequest,
        principal: &Principal,
        power: Power,
    ) -> CloudPhoneResult {
        let Some(provider) = self.provider(&request.provider_id).cloned() else {
            return refused(ui_text(CLOUD_PROVIDER_UNKNOWN_KEY));
        };
        let info = provider.describe();
        let (able, audit_key) = match power {
            Power::Start => (info.can_start, CLOUD_AUDIT_STARTED_KEY),
            Power::Stop => (info.can_stop, CLOUD_AUDIT_STOPPED_KEY),
        };
        if !able {
            return refused(ui_text(CLOUD_UNSUPPORTED_KEY));
        }
        let known = self
            .cached()
            .into_iter()
            .find(|phone| phone.provider_id == info.id && phone.id == request.phone_id);
        let target = known
            .as_ref()
            .map_or_else(|| request.phone_id.clone(), target_of);
        let id = request.phone_id.clone();

        self.manage(principal, target, audit_key, async move {
            let changed = match power {
                Power::Start => provider.start(&id),
                Power::Stop => provider.stop(&id),
            };
            tokio::time::timeout(POWER_TIMEOUT, changed)
                .await
                .unwrap_or_else(|_| Err(ui_text(CLOUD_TIMED_OUT_KEY)))
                .map(|phone| CloudPhone {
                    provider_id: info.id.clone(),
                    ..phone
                })
        })
        .await
    }

    pub async fn start(
        &self,
        request: CloudPhoneRequest,
        principal: &Principal,
    ) -> CloudPhoneResult {
        self.power(request, principal, Power::Start).await
    }

    pub async fn stop(
        &self,
        request: CloudPhoneRequest,
        principal: &Principal,
    ) -> CloudPhoneResult {
        self.power(request, principal, Power::Stop).await
    }

    pub async fn register(
        &self,
        request: RegisterCloudPhoneRequest,
        principal: &Principal,
    ) -> CloudPhoneResult {
        let name = request.name.trim().to_string();
        if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
            return refused(ui_text_with_params(
                CLOUD_NAME_INVALID_KEY,
                "",
                [("max", MAX_NAME_CHARS.to_string())],
            ));
        }
        let Ok(endpoint) = Endpoint::from_contract(request.endpoint.as_option()) else {
            return refused(ui_text(CLOUD_ENDPOINT_INVALID_KEY));
        };
        let endpoint = NetworkEndpoint {
            host: endpoint.host().to_string(),
            port: u32::from(endpoint.port()),
            ..Default::default()
        };
        let registered = Arc::clone(&self.registered);
        let target = format!("{}:{}", endpoint.host, endpoint.port);

        self.manage(principal, target, CLOUD_AUDIT_REGISTERED_KEY, async move {
            run_blocking(move || Ok(registered.register(&name, endpoint)))
                .await
                .map_err(storage_failed)?
        })
        .await
    }

    pub async fn remove(
        &self,
        request: CloudPhoneRequest,
        principal: &Principal,
    ) -> CloudPhoneResult {
        if request.provider_id != REGISTERED_PROVIDER_ID {
            return refused(ui_text(CLOUD_UNSUPPORTED_KEY));
        }
        let known = self.cached().into_iter().find(|phone| {
            phone.provider_id == REGISTERED_PROVIDER_ID && phone.id == request.phone_id
        });
        let target = known
            .as_ref()
            .map_or_else(|| request.phone_id.clone(), target_of);
        let registered = Arc::clone(&self.registered);
        let id = request.phone_id;

        self.manage(principal, target, CLOUD_AUDIT_REMOVED_KEY, async move {
            run_blocking(move || Ok(registered.remove(&id)))
                .await
                .map_err(storage_failed)?
        })
        .await
    }
}
