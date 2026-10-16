use tempfile::tempdir;

use std::fs;

use super::AndroidPlatformToolsInstallRecord;
use super::archive::validate_platform_tools_root;
use super::swap::{PendingAndroidPlatformToolsInstall, replace_install_root};

#[test]
fn validates_missing_platform_tools_root() {
    let dir = tempdir().expect("temp dir should exist");

    assert!(validate_platform_tools_root(dir.path()).is_err());
}

#[test]
fn completes_a_swap_after_an_interrupted_previous_swap() {
    let dir = tempdir().expect("temp dir should exist");
    let install_root = dir.path().join("platform-tools");
    let backup_root = dir.path().join("platform-tools.previous");
    let staged_root = dir.path().join("staged");
    fs::create_dir_all(&backup_root).expect("backup should exist");
    fs::write(backup_root.join("marker"), b"old").expect("old marker should save");
    fs::create_dir_all(&staged_root).expect("staged install should exist");
    fs::write(staged_root.join("marker"), b"new").expect("new marker should save");

    replace_install_root(&staged_root, &install_root, &backup_root).expect("swap should recover");
    PendingAndroidPlatformToolsInstall::new(
        AndroidPlatformToolsInstallRecord::default(),
        install_root.clone(),
        backup_root.clone(),
    )
    .commit();

    assert_eq!(
        fs::read(install_root.join("marker")).expect("new install should exist"),
        b"new"
    );
    assert!(!backup_root.exists());
}

#[test]
fn restores_a_backup_when_the_staged_directory_is_missing() {
    let dir = tempdir().expect("temp dir should exist");
    let install_root = dir.path().join("platform-tools");
    let backup_root = dir.path().join("platform-tools.previous");
    fs::create_dir_all(&backup_root).expect("backup should exist");
    fs::write(backup_root.join("marker"), b"old").expect("old marker should save");

    assert!(
        replace_install_root(&dir.path().join("missing"), &install_root, &backup_root).is_err()
    );
    assert_eq!(
        fs::read(install_root.join("marker")).expect("backup should be restored"),
        b"old"
    );
}

#[test]
fn drops_an_uncommitted_swap_back_to_the_previous_install() {
    let dir = tempdir().expect("temp dir should exist");
    let install_root = dir.path().join("platform-tools");
    let backup_root = dir.path().join("platform-tools.previous");
    let staged_root = dir.path().join("staged");
    fs::create_dir_all(&install_root).expect("install should exist");
    fs::write(install_root.join("marker"), b"old").expect("old marker should save");
    fs::create_dir_all(&staged_root).expect("staged install should exist");
    fs::write(staged_root.join("marker"), b"new").expect("new marker should save");

    replace_install_root(&staged_root, &install_root, &backup_root).expect("swap should start");
    drop(PendingAndroidPlatformToolsInstall::new(
        AndroidPlatformToolsInstallRecord::default(),
        install_root.clone(),
        backup_root,
    ));

    assert_eq!(
        fs::read(install_root.join("marker")).expect("old install should be restored"),
        b"old"
    );
}
