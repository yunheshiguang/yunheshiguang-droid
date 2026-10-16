use crate::workspace::adb::{AdbErrorKind, AdbProbeDiagnostic};
use crate::workspace::contract::{AdbDeviceActionResult, AdbErrorKindContract, DiagnosticCode};
use yunheshiguang_contract::KnownEnum;

pub(super) fn batch_diagnostic_code(results: &[AdbDeviceActionResult]) -> DiagnosticCode {
    if results
        .iter()
        .any(|result| result.error_kind == AdbErrorKindContract::PermissionDenied)
    {
        DiagnosticCode::AdbPermissionDenied
    } else if results
        .iter()
        .any(|result| result.error_kind == AdbErrorKindContract::Timeout)
    {
        DiagnosticCode::AdbCommandTimeout
    } else if results.iter().any(|result| {
        matches!(
            AdbErrorKindContract::known(result.error_kind),
            Some(AdbErrorKindContract::InvalidOutput | AdbErrorKindContract::OutputLimitExceeded)
        )
    }) {
        DiagnosticCode::AdbOutputInvalid
    } else if results
        .iter()
        .any(|result| result.error_kind == AdbErrorKindContract::Unsupported)
    {
        DiagnosticCode::AndroidSecurityLimit
    } else {
        DiagnosticCode::AdbConflict
    }
}

pub(super) fn diagnostic_error_kind(code: DiagnosticCode) -> AdbErrorKindContract {
    match code {
        DiagnosticCode::AndroidPlatformToolsAdbMissing => AdbErrorKindContract::NotFound,
        DiagnosticCode::AdbPermissionDenied | DiagnosticCode::AdbNoPermissions => {
            AdbErrorKindContract::PermissionDenied
        }
        DiagnosticCode::AdbCommandTimeout => AdbErrorKindContract::Timeout,
        DiagnosticCode::AdbOutputInvalid => AdbErrorKindContract::InvalidOutput,
        _ => AdbErrorKindContract::NonZeroExit,
    }
}

pub(super) fn diagnostic_message(diagnostic: &AdbProbeDiagnostic) -> String {
    diagnostic
        .message
        .params
        .get("message")
        .cloned()
        .filter(|message| !message.is_empty())
        .or_else(|| {
            (!diagnostic.message.fallback.is_empty()).then(|| diagnostic.message.fallback.clone())
        })
        .unwrap_or_else(|| format!("ADB is unavailable ({:?})", diagnostic.code))
}

pub(super) fn adb_error_kind(kind: AdbErrorKind) -> AdbErrorKindContract {
    match kind {
        AdbErrorKind::NotFound => AdbErrorKindContract::NotFound,
        AdbErrorKind::PermissionDenied => AdbErrorKindContract::PermissionDenied,
        AdbErrorKind::Timeout => AdbErrorKindContract::Timeout,
        AdbErrorKind::OutputLimitExceeded => AdbErrorKindContract::OutputLimitExceeded,
        AdbErrorKind::NonZeroExit => AdbErrorKindContract::NonZeroExit,
        AdbErrorKind::InvalidOutput => AdbErrorKindContract::InvalidOutput,
        AdbErrorKind::Io => AdbErrorKindContract::Io,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotes_precise_batch_diagnostics() {
        let results = vec![AdbDeviceActionResult {
            device_id: "device".to_string(),
            success: false,
            error_kind: AdbErrorKindContract::Timeout.into(),
            message: "timed out".to_string(),
            ..Default::default()
        }];

        assert_eq!(
            batch_diagnostic_code(&results),
            DiagnosticCode::AdbCommandTimeout
        );
    }
}
