use std::fs;
use std::path::{Path, PathBuf};

use super::failure::AndroidPlatformToolsInstallFailure;
use crate::workspace::android_tools::binary::binary_path;
use crate::workspace::tool_archive::ensure_executable;

pub(super) fn find_platform_tools_root(
    extract_root: &Path,
) -> Result<PathBuf, AndroidPlatformToolsInstallFailure> {
    let direct = extract_root.join("platform-tools");

    if direct.is_dir() {
        return validate_platform_tools_root(&direct).map(|()| direct);
    }

    fs::read_dir(extract_root)
        .map_err(AndroidPlatformToolsInstallFailure::archive)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| validate_platform_tools_root(path).is_ok())
        .ok_or_else(|| AndroidPlatformToolsInstallFailure::archive("platform-tools root not found"))
}

pub(super) fn validate_platform_tools_root(
    root: &Path,
) -> Result<(), AndroidPlatformToolsInstallFailure> {
    if !root.is_dir() {
        return Err(AndroidPlatformToolsInstallFailure::archive(
            "platform-tools directory is missing",
        ));
    }

    for binary in ["adb", "fastboot"] {
        let path = binary_path(root, binary);
        if !path.is_file() {
            return Err(AndroidPlatformToolsInstallFailure::archive(format!(
                "{binary} is missing from platform-tools"
            )));
        }
        ensure_executable(&path).map_err(AndroidPlatformToolsInstallFailure::install)?;
    }

    crate::workspace::adb::AdbClient::new(binary_path(root, "adb"))
        .version()
        .map_err(AndroidPlatformToolsInstallFailure::install)?;

    Ok(())
}
