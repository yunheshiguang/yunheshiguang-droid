use std::path::{Path, PathBuf};

use crate::workspace::contract::AndroidPlatformToolsPackageKind;

use super::super::package::package_kind_slug;

const STATE_FILE_NAME: &str = "android-platform-tools-state.json";
const INSTALL_DIR_NAME: &str = "install";
const STAGING_DIR_NAME: &str = "staging";

#[derive(Clone, Debug)]
pub struct StorageLayout {
    base_dir: PathBuf,
}

impl StorageLayout {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn state_path(&self) -> PathBuf {
        self.base_dir.join(STATE_FILE_NAME)
    }

    pub fn backup_path(&self) -> PathBuf {
        self.base_dir.join(format!("{STATE_FILE_NAME}.previous"))
    }

    pub fn install_root_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.base_dir
            .join(INSTALL_DIR_NAME)
            .join(package_kind_slug(kind))
    }

    pub fn install_backup_root_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.base_dir
            .join(INSTALL_DIR_NAME)
            .join(format!("{}.previous", package_kind_slug(kind)))
    }

    pub fn temp_dir_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.base_dir
            .join(STAGING_DIR_NAME)
            .join(package_kind_slug(kind))
    }
}

pub fn state_file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "state file".to_string())
}
