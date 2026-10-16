use super::spec::all_diagnostic_codes;
use super::*;
use crate::workspace::messages::*;

#[test]
fn maps_titles_and_recovery_texts() {
    let cases = [
        (
            DiagnosticCode::None,
            DIAGNOSTIC_TITLE_NONE_KEY,
            DIAGNOSTIC_RECOVERY_NONE_KEY,
        ),
        (
            DiagnosticCode::AdbConflict,
            DIAGNOSTIC_TITLE_ADB_CONFLICT_KEY,
            DIAGNOSTIC_RECOVERY_ADB_CONFLICT_KEY,
        ),
        (
            DiagnosticCode::DeviceOffline,
            DIAGNOSTIC_TITLE_DEVICE_OFFLINE_KEY,
            DIAGNOSTIC_RECOVERY_DEVICE_OFFLINE_KEY,
        ),
        (
            DiagnosticCode::RouterCongestion,
            DIAGNOSTIC_TITLE_ROUTER_CONGESTION_KEY,
            DIAGNOSTIC_RECOVERY_ROUTER_CONGESTION_KEY,
        ),
        (
            DiagnosticCode::UsbUnauthorized,
            DIAGNOSTIC_TITLE_USB_UNAUTHORIZED_KEY,
            DIAGNOSTIC_RECOVERY_USB_UNAUTHORIZED_KEY,
        ),
        (
            DiagnosticCode::VendorPermission,
            DIAGNOSTIC_TITLE_VENDOR_PERMISSION_KEY,
            DIAGNOSTIC_RECOVERY_VENDOR_PERMISSION_KEY,
        ),
        (
            DiagnosticCode::Webview2Missing,
            DIAGNOSTIC_TITLE_WEBVIEW2_MISSING_KEY,
            DIAGNOSTIC_RECOVERY_WEBVIEW2_MISSING_KEY,
        ),
        (
            DiagnosticCode::AndroidSecurityLimit,
            DIAGNOSTIC_TITLE_ANDROID_SECURITY_LIMIT_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_SECURITY_LIMIT_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsUnavailable,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_UNAVAILABLE_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_UNAVAILABLE_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsInvalidDirectory,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_INVALID_DIRECTORY_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_INVALID_DIRECTORY_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsAdbMissing,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsFastbootMissing,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_FASTBOOT_MISSING_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_FASTBOOT_MISSING_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsDownloadFailed,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_DOWNLOAD_FAILED_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_DOWNLOAD_FAILED_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsArchiveInvalid,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_ARCHIVE_INVALID_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_ARCHIVE_INVALID_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsInstallFailed,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_INSTALL_FAILED_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_INSTALL_FAILED_KEY,
        ),
        (
            DiagnosticCode::AndroidPlatformToolsStateRecovered,
            DIAGNOSTIC_TITLE_ANDROID_PLATFORM_TOOLS_STATE_RECOVERED_KEY,
            DIAGNOSTIC_RECOVERY_ANDROID_PLATFORM_TOOLS_STATE_RECOVERED_KEY,
        ),
    ];

    for (code, title, recovery) in cases {
        assert_eq!(title_key(code), title);
        assert_eq!(recovery_key(code), recovery);
    }
}

#[test]
fn maps_health_from_code() {
    assert_eq!(health_for(DiagnosticCode::None), ConnectionHealth::Ready);
    assert_eq!(
        health_for(DiagnosticCode::RouterCongestion),
        ConnectionHealth::Degraded
    );
    assert_eq!(
        health_for(DiagnosticCode::VendorPermission),
        ConnectionHealth::Degraded
    );
    assert_eq!(
        health_for(DiagnosticCode::UsbUnauthorized),
        ConnectionHealth::Blocked
    );
}

#[test]
fn only_the_none_code_is_healthy() {
    for code in all_diagnostic_codes() {
        let spec = spec_for(code);
        let expected_ready = code == DiagnosticCode::None;

        assert_eq!(
            spec.health == ConnectionHealth::Ready,
            expected_ready,
            "{code:?} has the wrong health"
        );
    }
}

#[test]
fn every_code_has_distinct_title_and_recovery_keys() {
    for code in all_diagnostic_codes() {
        let spec = spec_for(code);

        assert!(!spec.title_key.is_empty(), "{code:?} has no title key");
        assert!(
            !spec.recovery_key.is_empty(),
            "{code:?} has no recovery key"
        );
        assert_ne!(
            spec.title_key, spec.recovery_key,
            "{code:?} reuses one key for both the title and the recovery hint"
        );
    }
}
