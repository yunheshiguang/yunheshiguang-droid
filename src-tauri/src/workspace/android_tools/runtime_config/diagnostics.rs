use crate::workspace::android_tools::storage::AndroidPlatformToolsInstallRecord;
use crate::workspace::contract::{
    AndroidPlatformToolsInstallStatus, DiagnosticCode, DiagnosticSignal,
};
use crate::workspace::diagnostics;
use crate::workspace::messages::DIAGNOSTIC_IMPACT_RUNTIME_KEY;
use crate::workspace::text::ui_text_with_params;
use yunheshiguang_contract::KnownEnum;

fn signal(id: &str, code: DiagnosticCode, message: String) -> DiagnosticSignal {
    diagnostics::signal(
        id,
        code,
        ui_text_with_params(DIAGNOSTIC_IMPACT_RUNTIME_KEY, "", [("message", message)]),
    )
}

pub(super) fn install_failure_diagnostic(
    install: &AndroidPlatformToolsInstallRecord,
) -> Option<DiagnosticSignal> {
    if !matches!(install.status(), AndroidPlatformToolsInstallStatus::Failed) {
        return None;
    }

    let code = DiagnosticCode::known(install.last_error_code)
        .unwrap_or(DiagnosticCode::AndroidPlatformToolsInstallFailed);

    Some(signal(
        "android-platform-tools-install-failed",
        code,
        install.last_error.clone(),
    ))
}

pub(super) fn state_recovery_diagnostic(message: String) -> DiagnosticSignal {
    signal(
        "android-platform-tools-state-recovered",
        DiagnosticCode::AndroidPlatformToolsStateRecovered,
        message,
    )
}
