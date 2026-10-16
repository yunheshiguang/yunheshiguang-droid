mod migration;
mod paths;
mod record;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager, Runtime};
use tempfile::NamedTempFile;

use crate::workspace::contract::{
    AndroidPlatformToolsInstallStatus, AndroidPlatformToolsPackageKind,
};

pub use migration::default_state;
pub use record::{
    AndroidPlatformToolsInstallRecord, AndroidPlatformToolsStateRecord,
    ScopedAndroidPlatformToolsConfigRecord,
};

use migration::{MigrationRejection, migrate, settle_interrupted_install};
use paths::{StorageLayout, state_file_name};

#[derive(Clone, Debug)]
pub struct AndroidPlatformToolsStorage {
    pub(self) layout: StorageLayout,
    newer: Arc<Mutex<Vec<PathBuf>>>,
}

impl AndroidPlatformToolsStorage {
    pub fn from_app<R: Runtime>(app: &AppHandle<R>) -> io::Result<Self> {
        let base_dir = app
            .path()
            .app_local_data_dir()
            .map_err(io::Error::other)?
            .join("android-platform-tools");

        Ok(Self::new(base_dir))
    }

    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            layout: StorageLayout::new(base_dir),
            newer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn install_root_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.layout.install_root_for(kind)
    }

    pub fn install_backup_root_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.layout.install_backup_root_for(kind)
    }

    pub fn temp_dir_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.layout.temp_dir_for(kind)
    }

    #[cfg(test)]
    pub fn load_state(&self) -> AndroidPlatformToolsStateRecord {
        self.load_state_with_warning().0
    }

    pub fn load_state_with_warning(&self) -> (AndroidPlatformToolsStateRecord, Option<String>) {
        let mut failures = Vec::new();

        for path in [self.layout.state_path(), self.layout.backup_path()] {
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => {
                    failures.push(format!(
                        "{} could not be read: {error}",
                        state_file_name(&path)
                    ));
                    continue;
                }
            };

            let parsed = match yunheshiguang_contract::json::from_slice::<
                AndroidPlatformToolsStateRecord,
            >(&bytes)
            {
                Ok(state) => state,
                Err(error) => {
                    failures.push(format!("{} is invalid: {error}", state_file_name(&path)));
                    continue;
                }
            };

            match migrate(parsed) {
                Ok(state) => {
                    let warning = (!failures.is_empty()).then(|| {
                        format!(
                            "Recovered Android platform-tools settings after {}",
                            failures.join("; ")
                        )
                    });

                    return (settle_interrupted_install(state), warning);
                }
                Err(rejection) => {
                    let MigrationRejection::NewerSchema { .. } = rejection;
                    failures.push(format!(
                        "{} was left untouched: {}",
                        state_file_name(&path),
                        rejection.describe()
                    ));
                    self.newer
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(path.clone());
                }
            }
        }

        let warning = (!failures.is_empty()).then(|| {
            format!(
                "Android platform-tools settings were reset after {}",
                failures.join("; ")
            )
        });

        (default_state(), warning)
    }

    pub fn recover_install(&self, state: &AndroidPlatformToolsStateRecord) -> io::Result<()> {
        for kind in [
            AndroidPlatformToolsPackageKind::Windows,
            AndroidPlatformToolsPackageKind::Macos,
            AndroidPlatformToolsPackageKind::Linux,
        ] {
            let install_root = self.layout.install_root_for(kind);
            let backup_root = self.layout.install_backup_root_for(kind);

            if backup_root.exists() {
                if committed_swap(state, kind) && install_root.exists() {
                    fs::remove_dir_all(&backup_root)?;
                } else {
                    if install_root.exists() {
                        fs::remove_dir_all(&install_root)?;
                    }
                    fs::rename(&backup_root, &install_root)?;
                }
            } else if install_root.exists()
                && state.last_attempt.package_kind == kind as i32
                && state.install.status() != AndroidPlatformToolsInstallStatus::Installed
            {
                fs::remove_dir_all(&install_root)?;
            }
        }

        Ok(())
    }

    fn keep_newer_files(&self) -> io::Result<()> {
        let newer = std::mem::take(&mut *self.newer.lock().unwrap_or_else(PoisonError::into_inner));
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or_default();

        for path in newer.into_iter().filter(|path| path.exists()) {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            fs::rename(&path, path.with_file_name(format!("{name}.newer-{stamp}")))?;
        }
        Ok(())
    }

    pub fn save_state(&self, state: &AndroidPlatformToolsStateRecord) -> io::Result<()> {
        fs::create_dir_all(self.layout.base_dir())?;
        self.keep_newer_files()?;

        let bytes = serde_json::to_vec_pretty(state).map_err(io::Error::other)?;
        let mut temp = NamedTempFile::new_in(self.layout.base_dir())?;
        temp.write_all(&bytes)?;
        temp.flush()?;
        temp.as_file().sync_all()?;

        let path = self.layout.state_path();
        let backup = self.layout.backup_path();

        if path.exists() {
            if backup.exists() {
                fs::remove_file(&backup)?;
            }
            fs::rename(&path, &backup)?;
        }

        if let Err(error) = temp.persist(&path) {
            if backup.exists() {
                let _ = fs::rename(&backup, &path);
            }
            return Err(error.error);
        }

        if backup.exists() {
            let _ = fs::remove_file(backup);
        }

        Ok(())
    }
}

fn committed_swap(
    state: &AndroidPlatformToolsStateRecord,
    kind: AndroidPlatformToolsPackageKind,
) -> bool {
    state.install.status() == AndroidPlatformToolsInstallStatus::Installed
        && state.last_attempt.status() == AndroidPlatformToolsInstallStatus::Installed
        && state.install.package_kind == kind as i32
        && state.last_attempt.package_kind == kind as i32
        && !state.install.sha256.is_empty()
        && state.last_attempt.sha256 == state.install.sha256
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
