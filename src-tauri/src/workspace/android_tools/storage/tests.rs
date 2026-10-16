use std::path::Path;
use yunheshiguang_contract::KnownEnum;

use tempfile::tempdir;

use crate::workspace::contract::{
    AndroidPlatformToolsInstallStatus, AndroidPlatformToolsSource, DiagnosticCode,
};

use super::super::package::AndroidPlatformToolsPackage;
use super::migration::{MigrationRejection, STATE_SCHEMA_VERSION, settle_interrupted_install};
use super::*;

#[test]
fn persists_state_across_roundtrip() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let mut state = storage.load_state();

    state.window_configs.insert(
        "main".to_string(),
        ScopedAndroidPlatformToolsConfigRecord::from_contract(
            AndroidPlatformToolsSource::ManualDirectory,
            "C:/platform-tools".to_string(),
        ),
    );
    storage.save_state(&state).expect("state should save");

    let loaded = storage.load_state();
    assert_eq!(
        loaded
            .global_config
            .as_ref()
            .expect("legacy config should migrate")
            .platform_tools_root,
        "C:/platform-tools"
    );
    assert_eq!(
        loaded
            .global_config
            .expect("legacy config should migrate")
            .source(),
        AndroidPlatformToolsSource::ManualDirectory
    );
    assert!(loaded.window_configs.is_empty());
}

#[test]
fn recovers_from_a_corrupted_primary_state() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let mut state = default_state();
    state.global_config = Some(ScopedAndroidPlatformToolsConfigRecord::from_contract(
        AndroidPlatformToolsSource::ManualDirectory,
        "/opt/android/platform-tools".to_string(),
    ));
    fs::create_dir_all(storage.layout.base_dir()).expect("state directory should exist");
    fs::write(
        storage.layout.backup_path(),
        serde_json::to_vec(&state).expect("state should serialize"),
    )
    .expect("backup should save");
    fs::write(storage.layout.state_path(), b"{broken").expect("primary should save");

    let (loaded, warning) = storage.load_state_with_warning();

    assert_eq!(
        loaded
            .global_config
            .expect("backup config should load")
            .platform_tools_root,
        "/opt/android/platform-tools"
    );
    assert!(
        warning
            .as_deref()
            .is_some_and(|message| message.contains("Recovered"))
    );
}

#[test]
fn marks_an_interrupted_attempt_as_failed() {
    let package = AndroidPlatformToolsPackage::current_host()
        .expect("test host should have an official package");
    let mut state = default_state();
    state.last_attempt =
        AndroidPlatformToolsInstallRecord::installing(package, Path::new("platform-tools"));

    let normalized = settle_interrupted_install(state);

    assert_eq!(
        normalized.last_attempt.status(),
        AndroidPlatformToolsInstallStatus::Failed
    );
    assert_eq!(
        DiagnosticCode::known(normalized.last_attempt.last_error_code),
        Some(DiagnosticCode::AndroidPlatformToolsInstallFailed)
    );
}

#[test]
fn rolls_back_a_swap_without_a_committed_attempt() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let package = AndroidPlatformToolsPackage::current_host().expect("host package should exist");
    let install_root = storage.install_root_for(package.kind);
    let backup_root = storage.install_backup_root_for(package.kind);
    fs::create_dir_all(&install_root).expect("new install should exist");
    fs::write(install_root.join("marker"), b"new").expect("new marker should save");
    fs::create_dir_all(&backup_root).expect("backup should exist");
    fs::write(backup_root.join("marker"), b"old").expect("old marker should save");
    let mut state = default_state();
    state.install = AndroidPlatformToolsInstallRecord::installed(
        package,
        &install_root,
        &install_root,
        String::new(),
    );
    state.last_attempt = AndroidPlatformToolsInstallRecord::failed(
        package,
        &install_root,
        DiagnosticCode::AndroidPlatformToolsInstallFailed,
        "interrupted".to_string(),
    );

    storage
        .recover_install(&state)
        .expect("swap should roll back");

    assert_eq!(
        fs::read(install_root.join("marker")).expect("old install should return"),
        b"old"
    );
    assert!(!backup_root.exists());
}

#[test]
fn reports_the_install_time_as_a_timestamp() {
    let package = AndroidPlatformToolsPackage::current_host().expect("host package should exist");
    let root = Path::new("/opt/platform-tools");
    let installed =
        AndroidPlatformToolsInstallRecord::installed(package, root, root, "1760000000".to_string());

    assert_eq!(
        installed
            .into_contract()
            .installed_at
            .as_option()
            .map(|at| at.seconds),
        Some(1_760_000_000)
    );
    assert!(
        AndroidPlatformToolsInstallRecord::default()
            .into_contract()
            .installed_at
            .is_unset()
    );
}

#[test]
fn finishes_cleanup_for_a_committed_swap() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let package = AndroidPlatformToolsPackage::current_host().expect("host package should exist");
    let install_root = storage.install_root_for(package.kind);
    let backup_root = storage.install_backup_root_for(package.kind);
    fs::create_dir_all(&install_root).expect("new install should exist");
    fs::write(install_root.join("marker"), b"new").expect("new marker should save");
    fs::create_dir_all(&backup_root).expect("backup should exist");
    fs::write(backup_root.join("marker"), b"old").expect("old marker should save");
    let install = AndroidPlatformToolsInstallRecord::installed(
        package,
        &install_root,
        &install_root,
        String::new(),
    );
    let mut state = default_state();
    state.install = install.clone();
    state.last_attempt = install;

    storage
        .recover_install(&state)
        .expect("committed swap should finish");

    assert_eq!(
        fs::read(install_root.join("marker")).expect("new install should remain"),
        b"new"
    );
    assert!(!backup_root.exists());
}

#[test]
fn migrates_per_window_settings_into_the_global_slot() {
    let mut state = AndroidPlatformToolsStateRecord {
        schema_version: 1,
        ..AndroidPlatformToolsStateRecord::default()
    };
    state.window_configs.insert(
        "main".to_string(),
        ScopedAndroidPlatformToolsConfigRecord::from_contract(
            AndroidPlatformToolsSource::ManualDirectory,
            "/opt/platform-tools".to_string(),
        ),
    );

    let migrated = migrate(state).expect("version 1 should migrate");

    assert_eq!(migrated.schema_version, STATE_SCHEMA_VERSION);
    assert_eq!(
        migrated
            .global_config
            .expect("window config should move to the global slot")
            .platform_tools_root,
        "/opt/platform-tools"
    );
    assert!(migrated.window_configs.is_empty());
}

#[test]
fn refuses_to_rewrite_state_a_newer_build_wrote() {
    let state = AndroidPlatformToolsStateRecord {
        schema_version: STATE_SCHEMA_VERSION + 1,
        ..AndroidPlatformToolsStateRecord::default()
    };

    assert_eq!(
        migrate(state).err(),
        Some(MigrationRejection::NewerSchema {
            found: STATE_SCHEMA_VERSION + 1
        })
    );
}

#[test]
fn resets_instead_of_downgrading_state_from_a_newer_build() {
    let dir = tempdir().expect("temp dir should exist");
    let storage = AndroidPlatformToolsStorage::new(dir.path().to_path_buf());
    let future = AndroidPlatformToolsStateRecord {
        schema_version: STATE_SCHEMA_VERSION + 1,
        global_config: Some(ScopedAndroidPlatformToolsConfigRecord::from_contract(
            AndroidPlatformToolsSource::ManualDirectory,
            "/opt/platform-tools".to_string(),
        )),
        ..AndroidPlatformToolsStateRecord::default()
    };
    fs::create_dir_all(storage.layout.base_dir()).expect("state directory should exist");
    fs::write(
        storage.layout.state_path(),
        serde_json::to_vec(&future).expect("state should serialize"),
    )
    .expect("state should save");

    let (loaded, warning) = storage.load_state_with_warning();

    assert_eq!(loaded.schema_version, STATE_SCHEMA_VERSION);
    assert!(loaded.global_config.is_none());
    assert!(
        warning
            .as_deref()
            .is_some_and(|message| message.contains("schema version"))
    );

    storage.save_state(&loaded).expect("state should save");
    let kept = fs::read_dir(storage.layout.base_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".newer-"))
        .map(|entry| fs::read(entry.path()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        kept,
        [serde_json::to_vec(&future).unwrap()],
        "the newer build's settings are set aside, not overwritten"
    );
}
