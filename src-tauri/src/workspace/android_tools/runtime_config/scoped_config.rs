use std::path::PathBuf;
use yunheshiguang_contract::KnownEnum;

use crate::workspace::android_tools::storage::ScopedAndroidPlatformToolsConfigRecord;
use crate::workspace::contract::{AndroidPlatformToolsConfig, AndroidPlatformToolsSource};
use crate::workspace::tooling::path_to_string;

#[derive(Clone, Debug, Default)]
pub struct ScopedAndroidPlatformToolsConfig {
    pub source: AndroidPlatformToolsSource,
    pub platform_tools_root: PathBuf,
}

impl ScopedAndroidPlatformToolsConfig {
    pub fn from_contract(value: AndroidPlatformToolsConfig) -> Self {
        let source = AndroidPlatformToolsSource::known(value.source)
            .unwrap_or(AndroidPlatformToolsSource::Unspecified);

        Self {
            source,
            platform_tools_root: PathBuf::from(value.platform_tools_root.trim()),
        }
    }

    pub fn from_record(value: ScopedAndroidPlatformToolsConfigRecord) -> Self {
        Self {
            source: value.source(),
            platform_tools_root: PathBuf::from(value.platform_tools_root),
        }
    }

    pub fn into_record(self) -> ScopedAndroidPlatformToolsConfigRecord {
        ScopedAndroidPlatformToolsConfigRecord::from_contract(
            self.source,
            path_to_string(&self.platform_tools_root),
        )
    }

    pub fn into_contract(self) -> AndroidPlatformToolsConfig {
        AndroidPlatformToolsConfig {
            source: self.source.into(),
            platform_tools_root: path_to_string(&self.platform_tools_root),
            ..Default::default()
        }
    }
}
