use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::*;
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::local_state::{read_json, write_json};
use crate::workspace::messages::*;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::auth::Authenticators;
use super::gateway::{ApiGateway, MIN_PORT, effective_port};
use super::tokens::TokenStore;

pub struct ApiService {
    pub(super) gateway: ApiGateway,
    pub(super) tokens: Arc<TokenStore>,
    pub(super) authenticators: Arc<Authenticators>,
    pub(super) config_path: Option<PathBuf>,
    pub(super) config: Mutex<ApiGatewayConfig>,
    pub(super) governance: Arc<Governance>,
    pub(super) telemetry: Arc<Telemetry>,
}

pub fn load_config(path: &std::path::Path) -> ApiGatewayConfig {
    read_json(path).unwrap_or_default()
}

struct Change {
    audit: UiText,
    succeeded: bool,
    secret: String,
}

impl ApiService {
    fn config(&self) -> ApiGatewayConfig {
        self.config
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn state(&self) -> ApiGatewayState {
        let status = self.gateway.status();

        ApiGatewayState {
            config: self.config().into(),
            status: status.status.into(),
            address: status.address,
            problem: status.problem.into(),
            tokens: self.tokens.list(),
            authenticators: self.authenticators.describe(),
            ..Default::default()
        }
    }

    pub async fn restore(&self) {
        let config = self.config();

        if config.enabled {
            self.gateway.apply(&config).await;
        }
    }

    fn refused(&self, detail: UiText) -> ApiGatewayResult {
        ApiGatewayResult {
            accepted: false,
            detail: detail.into(),
            state: self.state().into(),
            secret: String::new(),
            ..Default::default()
        }
    }

    async fn manage(
        &self,
        principal: &Principal,
        work: impl Future<Output = Result<Change, UiText>>,
    ) -> ApiGatewayResult {
        let ticket = match self
            .governance
            .begin(GovernedAction::ManageApi, principal, &[])
        {
            Ok(ticket) => ticket,
            Err(reason) => return self.refused(reason),
        };
        let started = Instant::now();
        let result = match work.await {
            Ok(change) => {
                let outcome = if change.succeeded {
                    AuditOutcome::Succeeded
                } else {
                    AuditOutcome::Failed
                };
                ticket.finish(outcome, Some(change.audit));
                ApiGatewayResult {
                    accepted: true,
                    detail: Default::default(),
                    state: self.state().into(),
                    secret: change.secret,
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                self.refused(detail)
            }
        };

        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(
                GovernedAction::ManageApi,
                result.accepted,
                started.elapsed(),
            )
            .with_targets(1, u32::from(!result.accepted)),
        ));
        result
    }

    pub async fn update(
        &self,
        request: UpdateApiGatewayRequest,
        principal: &Principal,
    ) -> ApiGatewayResult {
        let config = request.config.into_option().unwrap_or_default();

        if config.port != 0 && !(MIN_PORT..=u32::from(u16::MAX)).contains(&config.port) {
            return self.refused(ui_text_with_params(
                API_PORT_INVALID_KEY,
                "",
                [("min", MIN_PORT.to_string())],
            ));
        }

        self.manage(principal, async {
            if let Some(path) = self.config_path.clone() {
                let saved = config.clone();
                run_blocking(move || write_json(&path, &saved).map_err(|error| error.to_string()))
                    .await
                    .map_err(|error| {
                        ui_text_with_params(API_STORAGE_FAILED_KEY, "", [("message", error)])
                    })?;
            }
            *self.config.lock().unwrap_or_else(PoisonError::into_inner) = config.clone();
            let status = self.gateway.apply(&config).await;

            Ok(Change {
                audit: if config.enabled {
                    status.problem.clone().unwrap_or_else(|| {
                        ui_text_with_params(
                            API_AUDIT_ENABLED_KEY,
                            "",
                            [("port", effective_port(&config).to_string())],
                        )
                    })
                } else {
                    ui_text(API_AUDIT_DISABLED_KEY)
                },
                succeeded: status.problem.is_none(),
                secret: String::new(),
            })
        })
        .await
    }

    pub async fn create_token(
        &self,
        request: CreateApiTokenRequest,
        principal: &Principal,
    ) -> ApiGatewayResult {
        let tokens = Arc::clone(&self.tokens);
        self.manage(principal, async move {
            let (token, secret) = run_blocking(move || Ok(tokens.create(&request.name)))
                .await
                .map_err(|error| {
                    ui_text_with_params(API_STORAGE_FAILED_KEY, "", [("message", error)])
                })??;

            Ok(Change {
                audit: ui_text_with_params(API_AUDIT_TOKEN_CREATED_KEY, "", [("name", token.name)]),
                succeeded: true,
                secret,
            })
        })
        .await
    }

    pub async fn revoke_token(
        &self,
        request: RevokeApiTokenRequest,
        principal: &Principal,
    ) -> ApiGatewayResult {
        let tokens = Arc::clone(&self.tokens);
        self.manage(principal, async move {
            let name = run_blocking(move || Ok(tokens.revoke(&request.id)))
                .await
                .map_err(|error| {
                    ui_text_with_params(API_STORAGE_FAILED_KEY, "", [("message", error)])
                })??
                .ok_or_else(|| ui_text(API_TOKEN_MISSING_KEY))?;

            Ok(Change {
                audit: ui_text_with_params(API_AUDIT_TOKEN_REVOKED_KEY, "", [("name", name)]),
                succeeded: true,
                secret: String::new(),
            })
        })
        .await
    }
}
