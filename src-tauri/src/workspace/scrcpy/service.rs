use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::adb::AdbService;
use crate::workspace::android_tools::{AndroidPlatformToolsRegistry, probe_android_tools};
use crate::workspace::clock;
use crate::workspace::contract::*;
use crate::workspace::governance::{AuditTicket, Governance, Principal};
use crate::workspace::local_state::{read_json, write_json};
use crate::workspace::telemetry::Telemetry;
use crate::workspace::tooling::path_to_string;

use super::args::{default_config, launch_args};
use super::diagnostics::ScrcpyProblem;
use super::install::{install, settle_staging};
use super::package::{RELEASE, ScrcpyPackage};
use super::resolve::{binaries, release_in, resolve};
use super::sessions::{Launch, Sessions};

const CONFIG_FILE: &str = "config.json";
const INSTALL_FILE: &str = "install.json";

struct Stored {
    config: ScrcpyConfig,
    install: ScrcpyInstall,
}

pub struct ScrcpyService {
    directory: PathBuf,
    stored: Mutex<Stored>,
    installing: Mutex<()>,
    sessions: Sessions,
    tools: Arc<AndroidPlatformToolsRegistry>,
    adb: Arc<AdbService>,
    governance: Arc<Governance>,
    telemetry: Arc<Telemetry>,
}

fn not_installed() -> ScrcpyInstall {
    ScrcpyInstall {
        status: ScrcpyInstallStatus::NotInstalled.into(),
        ..ScrcpyInstall::default()
    }
}

fn normalize(mut config: ScrcpyConfig) -> ScrcpyConfig {
    if config.source != ScrcpyBinarySource::ManualDirectory {
        config.scrcpy_root.clear();
    }
    config.scrcpy_root = config.scrcpy_root.trim().to_string();
    config.window_title = config.window_title.trim().to_string();
    config.crop = config.crop.trim().to_string();
    config
}

fn health_of(diagnostics: &[ScrcpyDiagnosticSignal]) -> ScrcpyConnectionHealth {
    let has =
        |health: ScrcpyConnectionHealth| diagnostics.iter().any(|signal| signal.health == health);

    if has(ScrcpyConnectionHealth::Blocked) {
        ScrcpyConnectionHealth::Blocked
    } else if has(ScrcpyConnectionHealth::Degraded) {
        ScrcpyConnectionHealth::Degraded
    } else {
        ScrcpyConnectionHealth::Ready
    }
}

impl ScrcpyService {
    pub fn new(
        directory: PathBuf,
        tools: Arc<AndroidPlatformToolsRegistry>,
        adb: Arc<AdbService>,
        governance: Arc<Governance>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        let config = read_json(&directory.join(CONFIG_FILE))
            .map(normalize)
            .unwrap_or_else(default_config);
        let mut install = read_json(&directory.join(INSTALL_FILE)).unwrap_or_else(not_installed);
        if install.status == ScrcpyInstallStatus::Installing {
            install.status = ScrcpyInstallStatus::Failed.into();
            install.last_error_code = ScrcpyDiagnosticCode::InstallFailed.into();
            install.last_error = "the previous install did not finish".to_string();
        }
        settle_staging(&directory, &directory.join(RELEASE));

        Self {
            directory,
            stored: Mutex::new(Stored { config, install }),
            installing: Mutex::new(()),
            sessions: Sessions::default(),
            tools,
            adb,
            governance,
            telemetry,
        }
    }

    fn stored(&self) -> MutexGuard<'_, Stored> {
        self.stored.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn managed_root(&self) -> PathBuf {
        self.directory.join(RELEASE)
    }

    fn adb_path(&self) -> Option<PathBuf> {
        probe_android_tools(&self.tools.config()).adb_path
    }

    pub fn state(&self, problem: Option<ScrcpyProblem>) -> ScrcpyState {
        let (config, install) = {
            let stored = self.stored();
            (stored.config.clone(), stored.install.clone())
        };
        let managed = self.managed_root();
        let resolved = resolve(&config, Some(&managed));
        let adb = self.adb_path();

        let mut problems = problem.into_iter().collect::<Vec<_>>();
        if let Err(missing) = &resolved {
            problems.push(missing.clone());
        }
        if adb.is_none() {
            problems.push(ScrcpyProblem::new(ScrcpyDiagnosticCode::AdbMissing, ""));
        }
        problems.dedup_by_key(|problem| problem.code);
        let diagnostics = problems
            .iter()
            .map(ScrcpyProblem::signal)
            .collect::<Vec<_>>();

        ScrcpyState {
            config: config.into(),
            binaries: binaries(&resolved.unwrap_or_default(), adb.as_deref()),
            adb_path: adb.as_deref().map(path_to_string).unwrap_or_default(),
            sessions: self.sessions.list(),
            health: health_of(&diagnostics).into(),
            diagnostics,
            install: install.into(),
            ..Default::default()
        }
    }

    fn finish(&self, ticket: AuditTicket, action: GovernedAction, started: Instant, ok: bool) {
        let outcome = if ok {
            AuditOutcome::Succeeded
        } else {
            AuditOutcome::Failed
        };
        ticket.finish(outcome, None);
        self.telemetry
            .record(TelemetryEvent::OperationFinished(OperationSample::new(
                action,
                ok,
                started.elapsed(),
            )));
    }

    fn save_config(&self, config: ScrcpyConfig) -> Result<(), ScrcpyProblem> {
        write_json(&self.directory.join(CONFIG_FILE), &config).map_err(|error| {
            ScrcpyProblem::new(ScrcpyDiagnosticCode::Unavailable, error.to_string())
        })?;
        self.stored().config = config;
        Ok(())
    }

    fn save_install(&self, install: ScrcpyInstall) {
        if let Err(error) = write_json(&self.directory.join(INSTALL_FILE), &install) {
            log::warn!("scrcpy: install record not saved: {error}");
        }
        self.stored().install = install;
    }

    pub fn update_config(&self, config: ScrcpyConfig, principal: &Principal) -> ScrcpyState {
        let action = GovernedAction::UpdateScrcpyConfig;
        let ticket = match self.governance.begin(action, principal, &[]) {
            Ok(ticket) => ticket,
            Err(reason) => return self.state(Some(ScrcpyProblem::denied(reason))),
        };
        let started = Instant::now();
        let saved = self.save_config(normalize(config));

        self.finish(ticket, action, started, saved.is_ok());
        self.state(saved.err())
    }

    pub fn install(&self, package_kind: i32, principal: &Principal) -> ScrcpyState {
        let action = GovernedAction::InstallScrcpy;
        let host = ScrcpyPackage::current_host();
        let requested = ScrcpyPackageKind::known(package_kind).and_then(ScrcpyPackage::from_kind);
        let package = match (host, requested) {
            (Some(host), Some(requested)) if host.kind != requested.kind => None,
            (host, _) => host,
        };
        let Some(package) = package else {
            return self.state(Some(ScrcpyProblem::new(
                ScrcpyDiagnosticCode::UnsupportedPackage,
                "",
            )));
        };
        let ticket = match self.governance.begin(action, principal, &[]) {
            Ok(ticket) => ticket,
            Err(reason) => return self.state(Some(ScrcpyProblem::denied(reason))),
        };
        if self.sessions.any_running() {
            self.finish(ticket, action, Instant::now(), false);
            return self.state(Some(ScrcpyProblem::new(
                ScrcpyDiagnosticCode::InstallFailed,
                "close every mirroring window before installing",
            )));
        }
        let Ok(_installing) = self.installing.try_lock() else {
            self.finish(ticket, action, Instant::now(), false);
            return self.state(Some(ScrcpyProblem::new(
                ScrcpyDiagnosticCode::InstallFailed,
                "another install is running",
            )));
        };
        let started = Instant::now();
        let record = |status: ScrcpyInstallStatus, root: &Path| ScrcpyInstall {
            status: status.into(),
            package_kind: package.kind.into(),
            release_tag: RELEASE.to_string(),
            install_root: path_to_string(root),
            source_url: package.archive.url.to_string(),
            ..ScrcpyInstall::default()
        };
        let root = self.managed_root();
        let previous = self.stored().install.clone();

        self.save_install(record(ScrcpyInstallStatus::Installing, &root));
        log::info!("scrcpy: installing {RELEASE} for {:?}", package.kind);
        let installed = install(package, &root).and_then(|root| {
            let config = ScrcpyConfig {
                source: ScrcpyBinarySource::AppPrivate.into(),
                ..self.stored().config.clone()
            };
            self.save_config(normalize(config)).map(|()| root)
        });

        let problem = match installed {
            Ok(root) => {
                self.save_install(ScrcpyInstall {
                    installed_at: clock::now().into(),
                    ..record(ScrcpyInstallStatus::Installed, &root)
                });
                None
            }
            Err(problem) => {
                log::error!("scrcpy: install failed: {}", problem.code.proto_name());
                let last_error = problem
                    .detail
                    .params
                    .get("message")
                    .cloned()
                    .unwrap_or_default();
                let kept = previous.status == ScrcpyInstallStatus::Installed
                    && release_in(&root).is_some();
                self.save_install(if kept {
                    ScrcpyInstall {
                        last_error,
                        last_error_code: problem.code.into(),
                        ..previous
                    }
                } else {
                    ScrcpyInstall {
                        last_error,
                        last_error_code: problem.code.into(),
                        ..record(ScrcpyInstallStatus::Failed, &root)
                    }
                });
                Some(problem)
            }
        };

        self.finish(ticket, action, started, problem.is_none());
        self.state(problem)
    }

    pub fn launch(
        &self,
        serial: &str,
        config: Option<ScrcpyConfig>,
        principal: &Principal,
        window_label: &str,
    ) -> ScrcpyState {
        let action = GovernedAction::MirrorDevice;
        let serial = serial.trim();
        let ticket = match self
            .governance
            .begin(action, principal, &[serial.to_string()])
        {
            Ok(ticket) => ticket,
            Err(reason) => return self.state(Some(ScrcpyProblem::denied(reason))),
        };
        let started = Instant::now();
        let Ok(_installing) = self.installing.try_lock() else {
            self.finish(ticket, action, started, false);
            return self.state(Some(ScrcpyProblem::new(
                ScrcpyDiagnosticCode::Unavailable,
                "scrcpy is being installed; try again when it finishes",
            )));
        };
        let lease = self.adb.lease();
        let stored = self.stored().config.clone();
        let options = config.map(normalize).unwrap_or_else(|| stored.clone());

        let launched = resolve(&stored, Some(&self.managed_root())).and_then(|resolved| {
            let adb = self
                .adb_path()
                .ok_or_else(|| ScrcpyProblem::new(ScrcpyDiagnosticCode::AdbMissing, ""))?;
            let client = resolved
                .client
                .ok_or_else(|| ScrcpyProblem::new(ScrcpyDiagnosticCode::BinaryMissing, ""))?;

            self.sessions.launch(Launch {
                client: &client,
                server: resolved.server.as_deref(),
                adb: &adb,
                args: launch_args(serial, &options)?,
                serial,
                window_label,
                lease,
            })
        });
        let problem = launched.err();

        self.finish(ticket, action, started, problem.is_none());
        self.state(problem)
    }

    pub fn stop_all(&self) {
        self.sessions.stop_all();
    }

    pub fn stop(&self, session_id: &str, serial: &str, principal: &Principal) -> ScrcpyState {
        let action = GovernedAction::MirrorDevice;
        let target = self
            .sessions
            .serial_of(session_id)
            .unwrap_or_else(|| serial.trim().to_string());
        if !self.sessions.contains(session_id, &target) {
            return self.state(None);
        }
        let ticket = match self
            .governance
            .begin(action, principal, std::slice::from_ref(&target))
        {
            Ok(ticket) => ticket,
            Err(reason) => return self.state(Some(ScrcpyProblem::denied(reason))),
        };
        let started = Instant::now();
        let stopped = self.sessions.stop(session_id, &target);

        self.finish(ticket, action, started, stopped.is_ok());
        self.state(stopped.err())
    }
}
