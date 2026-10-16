use std::collections::BTreeMap;
use std::path::Path;
use yunheshiguang_contract::KnownEnum;
use yunheshiguang_contract::wkt::Timestamp;

use serde::{Deserialize, Serialize};

use crate::workspace::contract::{
    AndroidPlatformToolsInstall, AndroidPlatformToolsInstallStatus,
    AndroidPlatformToolsPackageKind, AndroidPlatformToolsSource, DiagnosticCode,
};

use super::super::package::AndroidPlatformToolsPackage;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ScopedAndroidPlatformToolsConfigRecord {
    pub source: i32,
    pub platform_tools_root: String,
}

impl ScopedAndroidPlatformToolsConfigRecord {
    pub fn from_contract(source: AndroidPlatformToolsSource, platform_tools_root: String) -> Self {
        Self {
            source: source as i32,
            platform_tools_root: platform_tools_root.trim().to_string(),
        }
    }

    pub fn source(&self) -> AndroidPlatformToolsSource {
        AndroidPlatformToolsSource::known(self.source)
            .unwrap_or(AndroidPlatformToolsSource::Unspecified)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AndroidPlatformToolsInstallRecord {
    pub status: i32,
    pub package_kind: i32,
    pub install_root: String,
    pub platform_tools_root: String,
    pub source_url: String,
    pub installed_at: String,
    pub last_error: String,
    pub last_error_code: i32,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub sha256: String,
}

impl Default for AndroidPlatformToolsInstallRecord {
    fn default() -> Self {
        Self {
            status: AndroidPlatformToolsInstallStatus::NotInstalled as i32,
            package_kind: AndroidPlatformToolsPackageKind::Unspecified as i32,
            install_root: String::new(),
            platform_tools_root: String::new(),
            source_url: String::new(),
            installed_at: String::new(),
            last_error: String::new(),
            last_error_code: DiagnosticCode::None as i32,
            version: String::new(),
            sha256: String::new(),
        }
    }
}

impl AndroidPlatformToolsInstallRecord {
    pub fn installing(package: AndroidPlatformToolsPackage, install_root: &Path) -> Self {
        Self {
            status: AndroidPlatformToolsInstallStatus::Installing as i32,
            package_kind: package.kind as i32,
            install_root: install_root.to_string_lossy().to_string(),
            platform_tools_root: String::new(),
            source_url: package.url.to_string(),
            installed_at: String::new(),
            last_error: String::new(),
            last_error_code: DiagnosticCode::None as i32,
            version: package.version.to_string(),
            sha256: package.sha256.to_string(),
        }
    }

    pub fn installed(
        package: AndroidPlatformToolsPackage,
        install_root: &Path,
        platform_tools_root: &Path,
        installed_at: String,
    ) -> Self {
        Self {
            status: AndroidPlatformToolsInstallStatus::Installed as i32,
            package_kind: package.kind as i32,
            install_root: install_root.to_string_lossy().to_string(),
            platform_tools_root: platform_tools_root.to_string_lossy().to_string(),
            source_url: package.url.to_string(),
            installed_at,
            last_error: String::new(),
            last_error_code: DiagnosticCode::None as i32,
            version: package.version.to_string(),
            sha256: package.sha256.to_string(),
        }
    }

    pub fn failed(
        package: AndroidPlatformToolsPackage,
        install_root: &Path,
        error_code: DiagnosticCode,
        error: String,
    ) -> Self {
        Self {
            status: AndroidPlatformToolsInstallStatus::Failed as i32,
            package_kind: package.kind as i32,
            install_root: install_root.to_string_lossy().to_string(),
            platform_tools_root: String::new(),
            source_url: package.url.to_string(),
            installed_at: String::new(),
            last_error: error,
            last_error_code: error_code as i32,
            version: package.version.to_string(),
            sha256: package.sha256.to_string(),
        }
    }

    pub fn status(&self) -> AndroidPlatformToolsInstallStatus {
        AndroidPlatformToolsInstallStatus::known(self.status)
            .unwrap_or(AndroidPlatformToolsInstallStatus::Unspecified)
    }

    pub(super) fn interrupted(mut self) -> Self {
        self.status = AndroidPlatformToolsInstallStatus::Failed as i32;
        self.last_error_code = DiagnosticCode::AndroidPlatformToolsInstallFailed as i32;
        self.last_error = "the previous installation was interrupted".to_string();
        self
    }

    pub fn into_contract(self) -> AndroidPlatformToolsInstall {
        AndroidPlatformToolsInstall {
            status: self.status.into(),
            package_kind: self.package_kind.into(),
            install_root: self.install_root,
            platform_tools_root: self.platform_tools_root,
            source_url: self.source_url,
            installed_at: self
                .installed_at
                .parse()
                .ok()
                .map(|seconds| Timestamp {
                    seconds,
                    ..Default::default()
                })
                .into(),
            last_error: self.last_error,
            last_error_code: self.last_error_code.into(),
            version: self.version,
            sha256: self.sha256,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AndroidPlatformToolsStateRecord {
    pub schema_version: u32,
    #[serde(default)]
    pub global_config: Option<ScopedAndroidPlatformToolsConfigRecord>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub window_configs: BTreeMap<String, ScopedAndroidPlatformToolsConfigRecord>,
    pub install: AndroidPlatformToolsInstallRecord,
    #[serde(default)]
    pub last_attempt: AndroidPlatformToolsInstallRecord,
}
