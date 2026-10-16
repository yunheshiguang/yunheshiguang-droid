mod diagnostics;
mod scoped_config;

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, RwLock};
use yunheshiguang_contract::KnownEnum;

use tauri::{AppHandle, Runtime};

use crate::workspace::contract::{
    AndroidPlatformToolsInstall, AndroidPlatformToolsInstallStatus,
    AndroidPlatformToolsPackageKind, AndroidPlatformToolsSource, AndroidPlatformToolsState,
    DiagnosticSignal,
};
use crate::workspace::tooling::path_to_string;

use self::diagnostics::{install_failure_diagnostic, state_recovery_diagnostic};
use super::resolver::probe_android_tools;
use super::storage::{
    AndroidPlatformToolsInstallRecord, AndroidPlatformToolsStateRecord,
    AndroidPlatformToolsStorage, ScopedAndroidPlatformToolsConfigRecord,
};

pub use self::scoped_config::ScopedAndroidPlatformToolsConfig;

#[derive(Debug)]
pub struct AndroidPlatformToolsRegistry {
    storage: AndroidPlatformToolsStorage,
    state: RwLock<AndroidPlatformToolsStateRecord>,
    install_lock: Mutex<()>,
    load_warning: Option<String>,
}

impl AndroidPlatformToolsRegistry {
    pub fn from_app<R: Runtime>(app: &AppHandle<R>) -> io::Result<Self> {
        let storage = AndroidPlatformToolsStorage::from_app(app)?;
        let (state, mut load_warning) = storage.load_state_with_warning();
        if let Err(error) = storage.recover_install(&state) {
            log::error!("platform-tools: an interrupted install could not be recovered: {error}");
            load_warning.get_or_insert_with(|| {
                format!("an interrupted install could not be recovered: {error}")
            });
        }

        Ok(Self {
            storage,
            state: RwLock::new(state),
            install_lock: Mutex::new(()),
            load_warning,
        })
    }

    pub fn config(&self) -> ScopedAndroidPlatformToolsConfig {
        let state = self.state.read().unwrap_or_else(|err| err.into_inner());

        self.resolve_config(&state)
    }

    fn resolve_config(
        &self,
        state: &AndroidPlatformToolsStateRecord,
    ) -> ScopedAndroidPlatformToolsConfig {
        let installed = matches!(
            state.install.status(),
            AndroidPlatformToolsInstallStatus::Installed
        );
        let mut config = state
            .global_config
            .clone()
            .map(ScopedAndroidPlatformToolsConfig::from_record)
            .unwrap_or_else(|| ScopedAndroidPlatformToolsConfig {
                source: if installed {
                    AndroidPlatformToolsSource::AppPrivate
                } else {
                    AndroidPlatformToolsSource::SystemPath
                },
                platform_tools_root: PathBuf::new(),
            });

        if matches!(config.source, AndroidPlatformToolsSource::AppPrivate) {
            config.platform_tools_root =
                AndroidPlatformToolsPackageKind::known(state.install.package_kind)
                    .filter(|_| installed)
                    .map(|kind| self.storage.install_root_for(kind))
                    .unwrap_or_default();
        }

        config
    }

    pub fn install_contract(&self) -> AndroidPlatformToolsInstall {
        self.install_record().into_contract()
    }

    pub fn state(&self) -> AndroidPlatformToolsState {
        let config = self.config();
        let mut tools = probe_android_tools(&config);
        self.append_install_diagnostics(&mut tools.diagnostics);

        AndroidPlatformToolsState {
            config: config.into_contract().into(),
            adb_path: tools
                .adb_path
                .as_deref()
                .map(path_to_string)
                .unwrap_or_default(),
            fastboot_path: tools
                .fastboot_path
                .as_deref()
                .map(path_to_string)
                .unwrap_or_default(),
            binaries: tools.binaries(),
            diagnostics: tools.diagnostics.clone(),
            health: tools.health().into(),
            install: self.install_record().into_contract().into(),
            last_install_attempt: self.last_install_attempt_record().into_contract().into(),
            ..Default::default()
        }
    }

    pub fn install_record(&self) -> AndroidPlatformToolsInstallRecord {
        let state = self.state.read().unwrap_or_else(|err| err.into_inner());

        state.install.clone()
    }

    pub fn last_install_attempt_record(&self) -> AndroidPlatformToolsInstallRecord {
        let state = self.state.read().unwrap_or_else(|err| err.into_inner());

        state.last_attempt.clone()
    }

    pub fn append_install_diagnostics(&self, diagnostics: &mut Vec<DiagnosticSignal>) {
        if let Some(diagnostic) = install_failure_diagnostic(&self.last_install_attempt_record()) {
            diagnostics.push(diagnostic);
        }
        if let Some(message) = &self.load_warning {
            diagnostics.push(state_recovery_diagnostic(message.clone()));
        }
    }

    pub fn update_config(
        &self,
        config: ScopedAndroidPlatformToolsConfig,
    ) -> io::Result<AndroidPlatformToolsStateRecord> {
        self.mutate(|next| {
            next.global_config = Some(config.into_record());
            Ok(())
        })
    }

    pub fn update_install(
        &self,
        install: AndroidPlatformToolsInstallRecord,
    ) -> io::Result<AndroidPlatformToolsStateRecord> {
        self.mutate(|next| {
            if install.status() == AndroidPlatformToolsInstallStatus::Installed {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "installed records must be committed with complete_install",
                ));
            }

            next.last_attempt = install;
            Ok(())
        })
    }

    pub fn complete_install(
        &self,
        install: AndroidPlatformToolsInstallRecord,
    ) -> io::Result<AndroidPlatformToolsStateRecord> {
        self.mutate(|next| {
            if install.status() != AndroidPlatformToolsInstallStatus::Installed {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "only an installed Platform Tools record can be committed",
                ));
            }

            next.install = install.clone();
            next.last_attempt = install;
            next.global_config = Some(ScopedAndroidPlatformToolsConfigRecord::from_contract(
                AndroidPlatformToolsSource::AppPrivate,
                String::new(),
            ));
            Ok(())
        })
    }

    fn mutate(
        &self,
        change: impl FnOnce(&mut AndroidPlatformToolsStateRecord) -> io::Result<()>,
    ) -> io::Result<AndroidPlatformToolsStateRecord> {
        let mut state = self.state.write().unwrap_or_else(|err| err.into_inner());
        let mut next = state.clone();

        change(&mut next)?;
        self.storage.save_state(&next)?;
        *state = next.clone();

        Ok(next)
    }

    pub fn try_install_guard(&self) -> io::Result<MutexGuard<'_, ()>> {
        self.install_lock.try_lock().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock => io::Error::new(
                io::ErrorKind::AlreadyExists,
                "an installation is already running",
            ),
            std::sync::TryLockError::Poisoned(error) => {
                io::Error::other(format!("installation lock is poisoned: {error}"))
            }
        })
    }

    pub fn install_root_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.storage.install_root_for(kind)
    }

    pub fn temp_dir_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.storage.temp_dir_for(kind)
    }

    pub fn install_backup_root_for(&self, kind: AndroidPlatformToolsPackageKind) -> PathBuf {
        self.storage.install_backup_root_for(kind)
    }
}

#[cfg(test)]
mod tests;
