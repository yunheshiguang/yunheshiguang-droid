use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::failure::AndroidPlatformToolsInstallFailure;
use crate::workspace::android_tools::storage::AndroidPlatformToolsInstallRecord;
use crate::workspace::tool_archive::swap_directory;

pub struct PendingAndroidPlatformToolsInstall {
    record: AndroidPlatformToolsInstallRecord,
    install_root: PathBuf,
    backup_root: PathBuf,
    completed: bool,
}

impl PendingAndroidPlatformToolsInstall {
    pub(super) fn new(
        record: AndroidPlatformToolsInstallRecord,
        install_root: PathBuf,
        backup_root: PathBuf,
    ) -> Self {
        Self {
            record,
            install_root,
            backup_root,
            completed: false,
        }
    }

    pub fn record(&self) -> &AndroidPlatformToolsInstallRecord {
        &self.record
    }

    pub fn commit(mut self) {
        self.completed = true;

        if self.backup_root.exists() {
            let _ = fs::remove_dir_all(&self.backup_root);
        }
    }

    fn rollback(&mut self) -> io::Result<()> {
        if self.install_root.exists() {
            fs::remove_dir_all(&self.install_root)?;
        }
        if self.backup_root.exists() {
            fs::rename(&self.backup_root, &self.install_root)?;
        }
        self.completed = true;

        Ok(())
    }
}

impl Drop for PendingAndroidPlatformToolsInstall {
    fn drop(&mut self) {
        if !self.completed {
            let _ = self.rollback();
        }
    }
}

pub(super) fn prepare_directory(path: &Path) -> Result<(), AndroidPlatformToolsInstallFailure> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(AndroidPlatformToolsInstallFailure::install)?;
    }

    fs::create_dir_all(path).map_err(AndroidPlatformToolsInstallFailure::install)
}

pub(super) fn replace_install_root(
    staged_root: &Path,
    install_root: &Path,
    backup_root: &Path,
) -> Result<(), AndroidPlatformToolsInstallFailure> {
    swap_directory(staged_root, install_root, backup_root)
        .map_err(AndroidPlatformToolsInstallFailure::install)
}
