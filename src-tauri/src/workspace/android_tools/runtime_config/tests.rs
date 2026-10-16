use tempfile::tempdir;
use yunheshiguang_contract::KnownEnum;

use super::diagnostics::{install_failure_diagnostic, state_recovery_diagnostic};
use super::*;
use crate::workspace::contract::DiagnosticCode;

#[test]
fn resolves_default_config_from_state() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let registry = AndroidPlatformToolsRegistry {
        storage,
        state: RwLock::new(AndroidPlatformToolsStateRecord::default()),
        install_lock: Mutex::new(()),
        load_warning: None,
    };

    let config = registry.config();

    assert_eq!(config.source, AndroidPlatformToolsSource::SystemPath);
    assert!(config.platform_tools_root.as_os_str().is_empty());
}

#[test]
fn exposes_failed_install_as_diagnostic() {
    let install = AndroidPlatformToolsInstallRecord::failed(
        crate::workspace::android_tools::package::AndroidPlatformToolsPackage::current_host()
            .expect("test host should be supported"),
        std::path::Path::new("platform-tools"),
        DiagnosticCode::AndroidPlatformToolsDownloadFailed,
        "network error".to_string(),
    );

    let diagnostic = install_failure_diagnostic(&install).expect("diagnostic should exist");

    assert_eq!(
        DiagnosticCode::known(diagnostic.code),
        Some(DiagnosticCode::AndroidPlatformToolsDownloadFailed)
    );
}

#[test]
fn exposes_state_recovery_as_a_degraded_diagnostic() {
    let diagnostic = state_recovery_diagnostic("recovered backup".to_string());

    assert_eq!(
        DiagnosticCode::known(diagnostic.code),
        Some(DiagnosticCode::AndroidPlatformToolsStateRecovered)
    );
    assert_eq!(
        diagnostic.health,
        crate::workspace::contract::ConnectionHealth::Degraded
    );
}

#[test]
fn derives_the_private_runtime_path_from_native_storage() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let package =
        crate::workspace::android_tools::package::AndroidPlatformToolsPackage::current_host()
            .expect("test host should be supported");
    let expected = storage.install_root_for(package.kind);
    let mut state = AndroidPlatformToolsStateRecord {
        global_config: Some(ScopedAndroidPlatformToolsConfigRecord::from_contract(
            AndroidPlatformToolsSource::AppPrivate,
            String::new(),
        )),
        ..AndroidPlatformToolsStateRecord::default()
    };
    state.install = AndroidPlatformToolsInstallRecord::installed(
        package,
        std::path::Path::new("stale-metadata"),
        std::path::Path::new("stale-metadata"),
        String::new(),
    );
    let registry = AndroidPlatformToolsRegistry {
        storage,
        state: RwLock::new(state),
        install_lock: Mutex::new(()),
        load_warning: None,
    };

    assert_eq!(registry.config().platform_tools_root, expected);
}

#[test]
fn preserves_the_active_install_after_a_failed_update() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let package =
        crate::workspace::android_tools::package::AndroidPlatformToolsPackage::current_host()
            .expect("test host should be supported");
    let active = AndroidPlatformToolsInstallRecord::installed(
        package,
        std::path::Path::new("active"),
        std::path::Path::new("active"),
        String::new(),
    );
    let failed = AndroidPlatformToolsInstallRecord::failed(
        package,
        std::path::Path::new("candidate"),
        DiagnosticCode::AndroidPlatformToolsIntegrityFailed,
        "checksum mismatch".to_string(),
    );
    let state = AndroidPlatformToolsStateRecord {
        install: active.clone(),
        last_attempt: failed.clone(),
        ..AndroidPlatformToolsStateRecord::default()
    };
    let registry = AndroidPlatformToolsRegistry {
        storage,
        state: RwLock::new(state),
        install_lock: Mutex::new(()),
        load_warning: None,
    };

    assert_eq!(registry.install_record().status(), active.status());
    assert_eq!(
        registry.last_install_attempt_record().status(),
        failed.status()
    );
}
