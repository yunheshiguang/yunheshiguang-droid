use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::workspace::android_tools::AndroidToolEnvironment;
use crate::workspace::contract::{DiagnosticCode, UiText};
use crate::workspace::messages::{
    ADB_OFFLINE_KEY, ADB_UNAUTHORIZED_KEY, DIAGNOSTIC_IMPACT_RUNTIME_KEY,
};
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::client::AdbClient;
use super::device::{AdbDevice, AdbDeviceState};
use super::error::{AdbCommandError, AdbErrorKind};

#[derive(Clone, Debug)]
pub struct AdbEnvironment {
    pub version: Option<String>,
    pub devices: Vec<AdbDevice>,
    pub diagnostics: Vec<AdbProbeDiagnostic>,
}

#[derive(Clone, Debug)]
pub struct AdbProbeDiagnostic {
    pub code: DiagnosticCode,
    pub message: UiText,
}

const EXCLUSIVE_WAIT: Duration = Duration::from_secs(3);

#[derive(Debug, Default)]
pub struct AdbService {
    runtime_lock: Arc<RwLock<()>>,
}

impl AdbService {
    pub fn acquire_runtime(&self) -> RwLockReadGuard<'_, ()> {
        self.runtime_lock.read()
    }

    pub fn try_acquire_runtime_exclusive(&self) -> Option<RwLockWriteGuard<'_, ()>> {
        self.runtime_lock.try_write_for(EXCLUSIVE_WAIT)
    }

    pub fn lease(&self) -> Box<dyn Send + Sync> {
        Box::new(self.runtime_lock.read_arc())
    }

    pub fn probe_tools_locked(&self, tools: &AndroidToolEnvironment) -> AdbEnvironment {
        let Some(path) = tools.adb_path.as_deref() else {
            return AdbEnvironment {
                version: None,
                devices: Vec::new(),
                diagnostics: Vec::new(),
            };
        };

        self.probe_path_locked(Path::new(path))
    }

    pub fn probe_path_locked(&self, path: &Path) -> AdbEnvironment {
        let client = AdbClient::new(path);
        let version = match client.version() {
            Ok(version) => Some(version),
            Err(error) => return failed_environment(error),
        };

        match client.devices() {
            Ok(devices) => {
                let diagnostics = devices.iter().filter_map(device_diagnostic).collect();
                AdbEnvironment {
                    version,
                    devices,
                    diagnostics,
                }
            }
            Err(error) => failed_environment(error),
        }
    }

    pub fn diagnostic_for_device(&self, device: &AdbDevice) -> Option<AdbProbeDiagnostic> {
        device_diagnostic(device)
    }
}

fn device_diagnostic(device: &AdbDevice) -> Option<AdbProbeDiagnostic> {
    let code = device.state.facts().diagnostic;

    if code == DiagnosticCode::None {
        return None;
    }

    let message = match device.state {
        AdbDeviceState::Unauthorized => ui_text_with_params(
            ADB_UNAUTHORIZED_KEY,
            "",
            [("serial", device.serial.clone())],
        ),
        AdbDeviceState::Offline => ui_text(ADB_OFFLINE_KEY),
        AdbDeviceState::NoPermissions => runtime_message(
            "ADB cannot access this USB device; verify udev rules and group membership",
        ),
        _ => device.state.note(),
    };

    Some(AdbProbeDiagnostic { code, message })
}

fn failed_environment(error: AdbCommandError) -> AdbEnvironment {
    let code = match error.kind {
        AdbErrorKind::NotFound => DiagnosticCode::AndroidPlatformToolsAdbMissing,
        AdbErrorKind::PermissionDenied => DiagnosticCode::AdbPermissionDenied,
        AdbErrorKind::Timeout => DiagnosticCode::AdbCommandTimeout,
        AdbErrorKind::InvalidOutput | AdbErrorKind::OutputLimitExceeded => {
            DiagnosticCode::AdbOutputInvalid
        }
        _ => DiagnosticCode::AdbConflict,
    };

    AdbEnvironment {
        version: None,
        devices: Vec::new(),
        diagnostics: vec![AdbProbeDiagnostic {
            code,
            message: runtime_message(error.to_string()),
        }],
    }
}

fn runtime_message(message: impl Into<String>) -> UiText {
    ui_text_with_params(
        DIAGNOSTIC_IMPACT_RUNTIME_KEY,
        "",
        [("message", message.into())],
    )
}
