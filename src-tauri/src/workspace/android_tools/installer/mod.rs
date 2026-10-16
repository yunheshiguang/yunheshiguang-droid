mod archive;
mod failure;
mod swap;

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use std::path::PathBuf;

use reqwest::blocking::Client;
use tempfile::{Builder, TempDir};

use self::archive::find_platform_tools_root;
use self::swap::{prepare_directory, replace_install_root};
use super::package::AndroidPlatformToolsPackage;
use super::storage::AndroidPlatformToolsInstallRecord;
use crate::workspace::tool_archive::{extract, fetch, https_client};

pub use self::failure::AndroidPlatformToolsInstallFailure;
pub use self::swap::PendingAndroidPlatformToolsInstall;

pub struct AndroidPlatformToolsInstaller {
    client: Client,
}

impl AndroidPlatformToolsInstaller {
    pub fn new() -> Result<Self, AndroidPlatformToolsInstallFailure> {
        Ok(Self {
            client: https_client()?,
        })
    }

    pub fn stage(
        &self,
        package: AndroidPlatformToolsPackage,
        temp_root: &Path,
    ) -> Result<StagedPlatformTools, AndroidPlatformToolsInstallFailure> {
        if !package.supports_current_host() {
            return Err(AndroidPlatformToolsInstallFailure::unsupported_host(
                "the selected platform-tools package does not support this host",
            ));
        }

        prepare_directory(temp_root)?;
        let temp_dir = Builder::new()
            .prefix("platform-tools-")
            .tempdir_in(temp_root)
            .map_err(AndroidPlatformToolsInstallFailure::install)?;
        let archive_path = temp_dir.path().join("platform-tools.zip");
        let extract_root = temp_dir.path().join("extract");

        let archive = package.archive();
        fetch(&self.client, archive, &archive_path)?;
        extract(&archive_path, archive.format, &extract_root)?;

        let platform_tools_root = find_platform_tools_root(&extract_root)?;

        Ok(StagedPlatformTools {
            package,
            platform_tools_root,
            _temp_dir: temp_dir,
        })
    }
}

pub struct StagedPlatformTools {
    package: AndroidPlatformToolsPackage,
    platform_tools_root: PathBuf,
    _temp_dir: TempDir,
}

impl StagedPlatformTools {
    pub fn swap_into(
        self,
        install_root: &Path,
        backup_root: &Path,
    ) -> Result<PendingAndroidPlatformToolsInstall, AndroidPlatformToolsInstallFailure> {
        fs::create_dir_all(install_root.parent().unwrap_or(install_root))
            .map_err(AndroidPlatformToolsInstallFailure::install)?;
        replace_install_root(&self.platform_tools_root, install_root, backup_root)?;

        Ok(PendingAndroidPlatformToolsInstall::new(
            AndroidPlatformToolsInstallRecord::installed(
                self.package,
                install_root,
                install_root,
                installed_at(),
            ),
            install_root.to_path_buf(),
            backup_root.to_path_buf(),
        ))
    }
}

fn installed_at() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
