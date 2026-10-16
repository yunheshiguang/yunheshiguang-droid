use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::documents::DocumentStore;
use yunheshiguang_sdk::telemetry::TelemetryEvent;

use crate::workspace::adb::{AdbProbeDiagnostic, AdbService, DeviceIdentities};
use crate::workspace::android_tools::{
    AndroidPlatformToolsRegistry, adb_missing_text, probe_android_tools,
};
use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::*;
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::input::InputRouter;
use crate::workspace::messages::API_HOST_FILES_KEY;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};
use crate::workspace::scripts::ScriptRunner;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::ui_text;
use crate::workspace::vendors::VendorRegistry;

use super::actions::{ActionContext, handler};
use super::devices::{plan_devices, unavailable_result};
use super::outcome::{
    audit_outcome, blocked_result, denied_result, operation_sample, summarize, with_hints,
};
use super::report::log_outcome;
use super::spec::spec_for;
use super::task::{action_kind, runtime_error_detail, validate_request};

pub struct BatchService {
    governance: Arc<Governance>,
    telemetry: Arc<Telemetry>,
    registry: Arc<AndroidPlatformToolsRegistry>,
    adb_service: Arc<AdbService>,
    vendors: Arc<VendorRegistry>,
    inputs: Arc<InputRouter>,
    scripts: Arc<ScriptRunner>,
    identities: Arc<DeviceIdentities>,
    documents: Arc<dyn DocumentStore>,
}

impl BatchService {
    pub async fn run(
        self: &Arc<Self>,
        request: BatchActionRequest,
        principal: &Principal,
    ) -> BatchActionResult {
        let service = Arc::clone(self);
        let principal = principal.clone();
        let fallback = request.clone();

        tauri::async_runtime::spawn(async move { service.run_detached(request, &principal).await })
            .await
            .unwrap_or_else(|error| {
                log::error!("batch: run failed: {error}");
                blocked_result(
                    &fallback,
                    runtime_error_detail(error.to_string()),
                    DiagnosticCode::AdbConflict,
                )
            })
    }

    async fn run_detached(
        self: &Arc<Self>,
        request: BatchActionRequest,
        principal: &Principal,
    ) -> BatchActionResult {
        let slug = spec_for(action_kind(&request)).slug;
        let started = Instant::now();

        if let Some(detail) = validate_request(&request) {
            log::debug!("batch {slug}: rejected by request validation");
            return blocked_result(&request, detail, DiagnosticCode::AndroidSecurityLimit);
        }
        let ticket = match self.governance.begin(
            GovernedAction::RunBatchAction,
            principal,
            &request.device_ids,
        ) {
            Ok(ticket) => ticket,
            Err(reason) => {
                log::info!("batch {slug}: denied by governance");
                return denied_result(&request, reason);
            }
        };
        if principal.is_api()
            && spec_for(action_kind(&request))
                .requirement
                .uses_host_files()
        {
            log::info!("batch {slug}: refused host files for a local API caller");
            let reason = ui_text(API_HOST_FILES_KEY);
            ticket.finish(AuditOutcome::Denied, Some(reason.clone()));
            return denied_result(&request, reason);
        }

        let service = Arc::clone(self);
        let request_for_join_error = request.clone();
        let result = run_blocking(move || Ok(service.run_with_runtime(request)))
            .await
            .unwrap_or_else(|error| {
                log::error!("batch {slug}: worker failed: {error}");
                blocked_result(
                    &request_for_join_error,
                    runtime_error_detail(error),
                    DiagnosticCode::AdbConflict,
                )
            });

        let outcome = audit_outcome(&result);
        ticket.finish(
            outcome,
            result
                .task
                .as_option()
                .and_then(|task| task.title.clone().into_option()),
        );
        log_outcome(slug, &result, started.elapsed());
        self.telemetry
            .record(TelemetryEvent::OperationFinished(operation_sample(
                &request_for_join_error,
                &result,
                outcome == AuditOutcome::Succeeded,
                started.elapsed(),
            )));
        result
    }

    fn run_with_runtime(&self, request: BatchActionRequest) -> BatchActionResult {
        let _runtime = self.adb_service.acquire_runtime();
        let config = self.registry.config();
        let tools = probe_android_tools(&config);
        let Some(adb_path) = tools.adb_path else {
            return unavailable_result(
                &request,
                AdbProbeDiagnostic {
                    code: DiagnosticCode::AndroidPlatformToolsAdbMissing,
                    message: adb_missing_text(),
                },
            );
        };

        self.execute(request, &adb_path)
    }

    fn execute(&self, request: BatchActionRequest, adb_path: &Path) -> BatchActionResult {
        let env = self.adb_service.probe_path_locked(adb_path);

        if env.version.is_none() {
            let diagnostic =
                env.diagnostics
                    .first()
                    .cloned()
                    .unwrap_or_else(|| AdbProbeDiagnostic {
                        code: DiagnosticCode::AdbConflict,
                        message: runtime_error_detail("ADB probing failed without a diagnostic"),
                    });

            return unavailable_result(&request, diagnostic);
        }

        let context = ActionContext {
            adb_path,
            identities: &self.identities,
            inputs: &self.inputs,
            scripts: &self.scripts,
            documents: self.documents.as_ref(),
        };
        let kind = action_kind(&request);
        let handler = handler(kind);
        let prepared = match handler.prepare(&request, &context) {
            Ok(prepared) => prepared,
            Err(detail) => {
                return blocked_result(&request, detail, DiagnosticCode::AndroidSecurityLimit);
            }
        };

        let (runnable, mut device_results) = plan_devices(
            &env.devices,
            &request.device_ids,
            handler.allowed_states(),
            &self.adb_service,
        );
        let dispatched = !runnable.is_empty();
        let run = prepared.run(&context, runnable);

        device_results.extend(run.device_results);
        device_results.sort_by(|left, right| left.device_id.cmp(&right.device_id));
        let hints = self.vendors.hints(adb_path, kind, &device_results);

        let mut result = with_hints(summarize(request, device_results, dispatched), hints);
        result.input_routes = run.input_routes;
        result.script_outputs = run.script_outputs;
        result
    }
}

pub struct BatchModule;

impl<R: Runtime> WorkspaceModule<R> for BatchModule {
    fn id(&self) -> &'static str {
        "batch"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let service = BatchService {
            governance: dependency(app)?,
            telemetry: dependency(app)?,
            registry: dependency(app)?,
            adb_service: dependency(app)?,
            vendors: dependency(app)?,
            inputs: dependency(app)?,
            scripts: dependency(app)?,
            identities: dependency(app)?,
            documents: dependency::<dyn DocumentStore, R>(app)?,
        };

        app.manage(Arc::new(service));

        let rpc = super::rpc::BatchRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
