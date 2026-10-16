use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, ServiceRequest, ServiceResult,
};
use yunheshiguang_contract::yunheshiguang::workspace::v1::PlatformToolsService;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

#[cfg(windows)]
use crate::workspace::adb::AdbClient;
use crate::workspace::adb::AdbService;
use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::{
    AndroidPlatformToolsInstallStatus, AndroidPlatformToolsPackageKind, AndroidPlatformToolsSource,
    AndroidPlatformToolsState, AuditOutcome, DiagnosticCode, GovernedAction,
    InstallAndroidPlatformToolsRequest, UiText, UpdateAndroidPlatformToolsConfigRequest,
};
use crate::workspace::diagnostics::signal;
use crate::workspace::governance::{AuditTicket, Governance, Principal};
use crate::workspace::rpc::{failure, principal};
use crate::workspace::telemetry::Telemetry;

use super::installer::{AndroidPlatformToolsInstallFailure, AndroidPlatformToolsInstaller};
use super::package::AndroidPlatformToolsPackage;
use super::runtime_config::{AndroidPlatformToolsRegistry, ScopedAndroidPlatformToolsConfig};
use super::storage::AndroidPlatformToolsInstallRecord;

const RUNTIME_BUSY: &str = "devices are busy with another operation; try again when it finishes";

fn requested_package(kind: i32) -> Result<AndroidPlatformToolsPackage, String> {
    let current = AndroidPlatformToolsPackage::current_host().ok_or_else(|| {
        "the official platform-tools package does not support this host".to_string()
    })?;
    let requested = AndroidPlatformToolsPackageKind::known(kind)
        .and_then(AndroidPlatformToolsPackage::from_kind);

    match requested {
        Some(package) if package.kind != current.kind => {
            Err("the requested platform-tools package does not match this host".to_string())
        }
        Some(package) => Ok(package),
        None => Ok(current),
    }
}

fn normalize_config(config: ScopedAndroidPlatformToolsConfig) -> ScopedAndroidPlatformToolsConfig {
    match config.source {
        AndroidPlatformToolsSource::ManualDirectory => config,
        AndroidPlatformToolsSource::AppPrivate => ScopedAndroidPlatformToolsConfig {
            source: AndroidPlatformToolsSource::AppPrivate,
            platform_tools_root: PathBuf::new(),
        },
        _ => ScopedAndroidPlatformToolsConfig {
            source: AndroidPlatformToolsSource::SystemPath,
            platform_tools_root: PathBuf::new(),
        },
    }
}

fn denied_state(
    registry: &AndroidPlatformToolsRegistry,
    reason: UiText,
) -> AndroidPlatformToolsState {
    let mut state = registry.state();

    state.diagnostics.insert(
        0,
        signal(
            "governance-denied",
            DiagnosticCode::ActionNotAuthorized,
            reason,
        ),
    );
    state
}

fn finish(
    telemetry: &Telemetry,
    ticket: AuditTicket,
    action: GovernedAction,
    started: Instant,
    outcome: AuditOutcome,
) {
    ticket.finish(outcome, None);
    telemetry.record(TelemetryEvent::OperationFinished(OperationSample::new(
        action,
        outcome == AuditOutcome::Succeeded,
        started.elapsed(),
    )));
}

async fn update_config(
    tools: &PlatformToolsRpc,
    principal: Principal,
    request: UpdateAndroidPlatformToolsConfigRequest,
) -> Result<AndroidPlatformToolsState, String> {
    let registry = Arc::clone(&tools.registry);
    let adb_service = Arc::clone(&tools.adb);
    let governance = Arc::clone(&tools.governance);
    let ticket = match governance.begin(GovernedAction::UpdatePlatformToolsConfig, &principal, &[])
    {
        Ok(ticket) => ticket,
        Err(reason) => return Ok(denied_state(&registry, reason)),
    };
    let started = Instant::now();
    let config = request
        .config
        .map(ScopedAndroidPlatformToolsConfig::from_contract)
        .unwrap_or_default();
    let config = normalize_config(config);
    let source = config.source.proto_name();

    let result = run_blocking(move || {
        let _runtime = adb_service
            .try_acquire_runtime_exclusive()
            .ok_or_else(|| RUNTIME_BUSY.to_string())?;

        registry
            .update_config(config)
            .inspect_err(|error| log::error!("platform-tools source {source}: {error}"))
            .map_err(|error| error.to_string())?;
        log::info!("platform-tools source set to {source}");

        Ok(registry.state())
    })
    .await;

    let outcome = if result.is_ok() {
        AuditOutcome::Succeeded
    } else {
        AuditOutcome::Failed
    };
    finish(
        &tools.telemetry,
        ticket,
        GovernedAction::UpdatePlatformToolsConfig,
        started,
        outcome,
    );
    result
}

async fn install(
    tools: &PlatformToolsRpc,
    principal: Principal,
    package: AndroidPlatformToolsPackage,
) -> Result<AndroidPlatformToolsState, String> {
    let registry = Arc::clone(&tools.registry);
    let adb_service = Arc::clone(&tools.adb);
    let governance = Arc::clone(&tools.governance);
    let ticket = match governance.begin(GovernedAction::InstallPlatformTools, &principal, &[]) {
        Ok(ticket) => ticket,
        Err(reason) => return Ok(denied_state(&registry, reason)),
    };
    let started = Instant::now();
    let install_root = registry.install_root_for(package.kind);
    let backup_root = registry.install_backup_root_for(package.kind);
    let temp_root = registry.temp_dir_for(package.kind);

    let result = run_blocking(move || {
        let _guard = registry
            .try_install_guard()
            .map_err(|error| error.to_string())?;
        registry
            .update_install(AndroidPlatformToolsInstallRecord::installing(
                package,
                &install_root,
            ))
            .map_err(|error| error.to_string())?;

        log::info!("installing platform-tools package {:?}", package.kind);
        let staged = AndroidPlatformToolsInstaller::new()
            .and_then(|installer| installer.stage(package, &temp_root));
        let lease = staged
            .is_ok()
            .then(|| adb_service.try_acquire_runtime_exclusive())
            .flatten();
        let install_result = match staged {
            Err(failure) => Err(failure),
            Ok(_) if lease.is_none() => {
                Err(AndroidPlatformToolsInstallFailure::install(RUNTIME_BUSY))
            }
            Ok(staged) => {
                stop_managed_adb(&install_root);
                staged.swap_into(&install_root, &backup_root)
            }
        };
        match install_result {
            Ok(pending) => {
                let install = pending.record().clone();
                match registry.complete_install(install) {
                    Ok(_) => {
                        pending.commit();
                        log::info!("platform-tools package {:?} installed", package.kind);
                    }
                    Err(error) => {
                        log::error!("platform-tools install could not be recorded: {error}");
                        drop(pending);
                        registry
                            .update_install(AndroidPlatformToolsInstallRecord::failed(
                                package,
                                &install_root,
                                DiagnosticCode::AndroidPlatformToolsInstallFailed,
                                error.to_string(),
                            ))
                            .map_err(|save_error| save_error.to_string())?;
                    }
                }
            }
            Err(error) => {
                log::error!(
                    "platform-tools install failed: {}: {}",
                    error.code.proto_name(),
                    error.message
                );
                registry
                    .update_install(AndroidPlatformToolsInstallRecord::failed(
                        package,
                        &registry.install_root_for(package.kind),
                        error.code,
                        error.message,
                    ))
                    .map_err(|save_error| save_error.to_string())?;
            }
        }

        Ok(registry.state())
    })
    .await;

    let installed = result.as_ref().is_ok_and(|state| {
        state
            .last_install_attempt
            .as_option()
            .is_some_and(|attempt| attempt.status == AndroidPlatformToolsInstallStatus::Installed)
    });
    let outcome = if installed {
        AuditOutcome::Succeeded
    } else {
        AuditOutcome::Failed
    };
    finish(
        &tools.telemetry,
        ticket,
        GovernedAction::InstallPlatformTools,
        started,
        outcome,
    );
    result
}

#[cfg(windows)]
fn stop_managed_adb(install_root: &std::path::Path) {
    let adb_path = crate::workspace::tooling::binary_path(install_root, "adb");
    if adb_path.is_file()
        && let Err(error) = AdbClient::new(adb_path).kill_server()
    {
        log::warn!("could not stop the managed ADB server before reinstalling: {error}");
    }
}

#[cfg(not(windows))]
fn stop_managed_adb(_install_root: &std::path::Path) {}

pub struct PlatformToolsRpc {
    pub registry: Arc<AndroidPlatformToolsRegistry>,
    pub adb: Arc<AdbService>,
    pub governance: Arc<Governance>,
    pub telemetry: Arc<Telemetry>,
}

impl PlatformToolsService for PlatformToolsRpc {
    async fn update_android_platform_tools_config(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, UpdateAndroidPlatformToolsConfigRequest>,
    ) -> ServiceResult<AndroidPlatformToolsState> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(
            update_config(self, principal, request)
                .await
                .map_err(failure)?,
        )
    }

    async fn install_android_platform_tools(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, InstallAndroidPlatformToolsRequest>,
    ) -> ServiceResult<AndroidPlatformToolsState> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        if !request.license_accepted {
            return Err(ConnectError::new(
                ErrorCode::FailedPrecondition,
                "the Android SDK license must be accepted before installation",
            ));
        }
        let package = requested_package(request.package_kind.to_i32())
            .map_err(|message| ConnectError::new(ErrorCode::FailedPrecondition, message))?;

        Response::ok(install(self, principal, package).await.map_err(failure)?)
    }
}
