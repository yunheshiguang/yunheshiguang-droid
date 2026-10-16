use crate::workspace::contract::{ConnectionHealth, DiagnosticCode};
use crate::workspace::messages::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DiagnosticSpec {
    pub health: ConnectionHealth,
    pub title_key: &'static str,
    pub recovery_key: &'static str,
}

impl DiagnosticSpec {
    const fn new(
        health: ConnectionHealth,
        title_key: &'static str,
        recovery_key: &'static str,
    ) -> Self {
        Self {
            health,
            title_key,
            recovery_key,
        }
    }
}

pub(super) fn spec_for(code: DiagnosticCode) -> DiagnosticSpec {
    use ConnectionHealth::{
        CONNECTION_HEALTH_BLOCKED as Blocked, CONNECTION_HEALTH_DEGRADED as Degraded,
        CONNECTION_HEALTH_READY as Ready,
    };

    match code {
        DiagnosticCode::None => DiagnosticSpec::new(
            Ready,
            DIAGNOSTIC_TITLE_NONE_KEY,
            DIAGNOSTIC_RECOVERY_NONE_KEY,
        ),
        DiagnosticCode::RouterCongestion => DiagnosticSpec::new(
            Degraded,
            DIAGNOSTIC_TITLE_ROUTER_CONGESTION_KEY,
            DIAGNOSTIC_RECOVERY_ROUTER_CONGESTION_KEY,
        ),
        DiagnosticCode::VendorPermission => DiagnosticSpec::new(
            Degraded,
            DIAGNOSTIC_TITLE_VENDOR_PERMISSION_KEY,
            DIAGNOSTIC_RECOVERY_VENDOR_PERMISSION_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsFastbootMissing => DiagnosticSpec::new(
            Degraded,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_FASTBOOT_MISSING_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_FASTBOOT_MISSING_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsStateRecovered => DiagnosticSpec::new(
            Degraded,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_STATE_RECOVERED_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_STATE_RECOVERED_KEY,
        ),
        DiagnosticCode::AdbConflict
        | DiagnosticCode::AdbCommandTimeout
        | DiagnosticCode::AdbOutputInvalid => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ADB_CONFLICT_KEY,
            DIAGNOSTIC_RECOVERY_ADB_CONFLICT_KEY,
        ),
        DiagnosticCode::AdbPermissionDenied | DiagnosticCode::AdbNoPermissions => {
            DiagnosticSpec::new(
                Blocked,
                DIAGNOSTIC_TITLE_VENDOR_PERMISSION_KEY,
                DIAGNOSTIC_RECOVERY_VENDOR_PERMISSION_KEY,
            )
        }
        DiagnosticCode::Webview2Missing => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_WEBVIEW2_MISSING_KEY,
            DIAGNOSTIC_RECOVERY_WEBVIEW2_MISSING_KEY,
        ),
        DiagnosticCode::UsbUnauthorized => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_USB_UNAUTHORIZED_KEY,
            DIAGNOSTIC_RECOVERY_USB_UNAUTHORIZED_KEY,
        ),
        DiagnosticCode::AndroidSecurityLimit => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_SECURITY_LIMIT_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_SECURITY_LIMIT_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsUnavailable
        | DiagnosticCode::AndroidPlatformToolsHostUnsupported => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_UNAVAILABLE_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_UNAVAILABLE_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsInvalidDirectory => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_INVALID_DIRECTORY_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_INVALID_DIRECTORY_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsAdbMissing => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsDownloadFailed => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_DOWNLOAD_FAILED_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_DOWNLOAD_FAILED_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsArchiveInvalid
        | DiagnosticCode::AndroidPlatformToolsIntegrityFailed => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_ARCHIVE_INVALID_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_ARCHIVE_INVALID_KEY,
        ),
        DiagnosticCode::AndroidPlatformToolsInstallFailed => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_INSTALL_FAILED_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_INSTALL_FAILED_KEY,
        ),
        DiagnosticCode::DeviceOffline => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_DEVICE_OFFLINE_KEY,
            DIAGNOSTIC_RECOVERY_DEVICE_OFFLINE_KEY,
        ),
        DiagnosticCode::ActionNotAuthorized => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_ACTION_NOT_AUTHORIZED_KEY,
            DIAGNOSTIC_RECOVERY_ACTION_NOT_AUTHORIZED_KEY,
        ),
        DiagnosticCode::Unspecified => DiagnosticSpec::new(
            Blocked,
            DIAGNOSTIC_TITLE_UNKNOWN_KEY,
            DIAGNOSTIC_RECOVERY_UNKNOWN_KEY,
        ),
    }
}

#[cfg(test)]
pub(super) fn all_diagnostic_codes() -> impl Iterator<Item = DiagnosticCode> {
    use yunheshiguang_contract::Enumeration;

    DiagnosticCode::values().iter().copied()
}
