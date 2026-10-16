use crate::workspace::contract::{
    ScrcpyConnectionHealth, ScrcpyDiagnosticCode, ScrcpyDiagnosticSignal, ScrcpyUiText, UiText,
};
use crate::workspace::messages::*;
use yunheshiguang_contract::Enumeration;

#[derive(Clone, Debug, PartialEq)]
pub struct ScrcpyProblem {
    pub code: ScrcpyDiagnosticCode,
    pub detail: ScrcpyUiText,
}

impl ScrcpyProblem {
    pub fn new(code: ScrcpyDiagnosticCode, message: impl Into<String>) -> Self {
        let message = message.into();
        let detail = if message.is_empty() {
            text(DIAGNOSTIC_IMPACT_DEFAULT_KEY, [])
        } else {
            text(DIAGNOSTIC_IMPACT_RUNTIME_KEY, [("message", message)])
        };

        Self { code, detail }
    }

    pub fn denied(reason: UiText) -> Self {
        Self {
            code: ScrcpyDiagnosticCode::NotAuthorized,
            detail: ScrcpyUiText {
                key: reason.key,
                params: reason.params,
                fallback: reason.fallback,
                ..Default::default()
            },
        }
    }

    pub fn signal(&self) -> ScrcpyDiagnosticSignal {
        let (title, recovery, health) = describe(self.code);

        ScrcpyDiagnosticSignal {
            id: format!("scrcpy-{}", self.code.proto_name().to_ascii_lowercase()),
            code: self.code.into(),
            health: health.into(),
            title: text(title, []).into(),
            impact: self.detail.clone().into(),
            recovery: text(recovery, []).into(),
            ..Default::default()
        }
    }
}

fn text<const N: usize>(key: &str, params: [(&str, String); N]) -> ScrcpyUiText {
    ScrcpyUiText {
        key: key.to_string(),
        params: params
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
        fallback: String::new(),
        ..Default::default()
    }
}

fn describe(code: ScrcpyDiagnosticCode) -> (&'static str, &'static str, ScrcpyConnectionHealth) {
    use ScrcpyConnectionHealth::{
        SCRCPY_CONNECTION_HEALTH_BLOCKED as Blocked, SCRCPY_CONNECTION_HEALTH_DEGRADED as Degraded,
    };
    use ScrcpyDiagnosticCode as Code;

    match code {
        Code::Unspecified | Code::None | Code::Unavailable => (
            SCRCPY_DIAGNOSTIC_TITLE_UNAVAILABLE_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_UNAVAILABLE_KEY,
            Blocked,
        ),
        Code::InvalidDirectory => (
            SCRCPY_DIAGNOSTIC_TITLE_INVALID_DIRECTORY_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_INVALID_DIRECTORY_KEY,
            Blocked,
        ),
        Code::BinaryMissing => (
            SCRCPY_DIAGNOSTIC_TITLE_BINARY_MISSING_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_BINARY_MISSING_KEY,
            Blocked,
        ),
        Code::AdbUnavailable => (
            SCRCPY_DIAGNOSTIC_TITLE_ADB_UNAVAILABLE_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_ADB_UNAVAILABLE_KEY,
            Blocked,
        ),
        Code::AdbMissing => (
            SCRCPY_DIAGNOSTIC_TITLE_ADB_MISSING_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_ADB_MISSING_KEY,
            Blocked,
        ),
        Code::InvalidSerial => (
            SCRCPY_DIAGNOSTIC_TITLE_INVALID_SERIAL_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_INVALID_SERIAL_KEY,
            Degraded,
        ),
        Code::InvalidArgument => (
            SCRCPY_DIAGNOSTIC_TITLE_INVALID_ARGUMENT_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_INVALID_ARGUMENT_KEY,
            Degraded,
        ),
        Code::AlreadyRunning => (
            SCRCPY_DIAGNOSTIC_TITLE_ALREADY_RUNNING_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_ALREADY_RUNNING_KEY,
            Degraded,
        ),
        Code::LaunchFailed => (
            SCRCPY_DIAGNOSTIC_TITLE_LAUNCH_FAILED_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_LAUNCH_FAILED_KEY,
            Degraded,
        ),
        Code::StopFailed => (
            SCRCPY_DIAGNOSTIC_TITLE_STOP_FAILED_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_STOP_FAILED_KEY,
            Degraded,
        ),
        Code::ChecksumMismatch => (
            SCRCPY_DIAGNOSTIC_TITLE_CHECKSUM_MISMATCH_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_CHECKSUM_MISMATCH_KEY,
            Blocked,
        ),
        Code::DownloadFailed => (
            SCRCPY_DIAGNOSTIC_TITLE_DOWNLOAD_FAILED_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_DOWNLOAD_FAILED_KEY,
            Blocked,
        ),
        Code::ArchiveInvalid => (
            SCRCPY_DIAGNOSTIC_TITLE_ARCHIVE_INVALID_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_ARCHIVE_INVALID_KEY,
            Blocked,
        ),
        Code::InstallFailed => (
            SCRCPY_DIAGNOSTIC_TITLE_INSTALL_FAILED_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_INSTALL_FAILED_KEY,
            Blocked,
        ),
        Code::UnsupportedPackage => (
            SCRCPY_DIAGNOSTIC_TITLE_UNSUPPORTED_PACKAGE_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_UNSUPPORTED_PACKAGE_KEY,
            Blocked,
        ),
        Code::NotAuthorized => (
            SCRCPY_DIAGNOSTIC_TITLE_NOT_AUTHORIZED_KEY,
            SCRCPY_DIAGNOSTIC_RECOVERY_NOT_AUTHORIZED_KEY,
            Degraded,
        ),
    }
}
