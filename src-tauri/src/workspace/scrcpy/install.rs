use std::fs;
use std::path::{Path, PathBuf};

use crate::workspace::contract::ScrcpyDiagnosticCode;
use crate::workspace::tool_archive::{
    ArchiveError, ensure_executable, extract, fetch, https_client, swap_directory,
};

pub const STAGING_PREFIX: &str = ".scrcpy-";
const PREVIOUS: &str = "previous";
use crate::workspace::tooling::binary_path;

use super::diagnostics::ScrcpyProblem;
use super::package::ScrcpyPackage;
use super::resolve::{CLIENT, release_in};

impl From<ArchiveError> for ScrcpyProblem {
    fn from(error: ArchiveError) -> Self {
        match error {
            ArchiveError::Download(message) => {
                Self::new(ScrcpyDiagnosticCode::DownloadFailed, message)
            }
            ArchiveError::Integrity(message) => {
                Self::new(ScrcpyDiagnosticCode::ChecksumMismatch, message)
            }
            ArchiveError::Archive(message) => {
                Self::new(ScrcpyDiagnosticCode::ArchiveInvalid, message)
            }
        }
    }
}

fn install_failed(error: impl ToString) -> ScrcpyProblem {
    ScrcpyProblem::new(ScrcpyDiagnosticCode::InstallFailed, error.to_string())
}

fn find_release(extracted: &Path) -> Option<PathBuf> {
    if release_in(extracted).is_some() {
        return Some(extracted.to_path_buf());
    }

    fs::read_dir(extracted)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| release_in(path).is_some())
}

pub fn settle_staging(directory: &Path, install_root: &Path) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for staging in entries.flatten().map(|entry| entry.path()) {
        let staged = staging
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(STAGING_PREFIX));
        if !staged {
            continue;
        }
        let previous = staging.join(PREVIOUS);
        if !install_root.exists()
            && release_in(&previous).is_some()
            && let Err(error) = fs::rename(&previous, install_root)
        {
            log::warn!("scrcpy: the previous release was not put back: {error}");
        }
        if let Err(error) = fs::remove_dir_all(&staging) {
            log::warn!("scrcpy: staging folder not removed: {error}");
        }
    }
}

pub fn install(package: ScrcpyPackage, install_root: &Path) -> Result<PathBuf, ScrcpyProblem> {
    let parent = install_root.parent().unwrap_or(install_root);
    fs::create_dir_all(parent).map_err(install_failed)?;
    let staging = tempfile::Builder::new()
        .prefix(STAGING_PREFIX)
        .tempdir_in(parent)
        .map_err(install_failed)?;
    let archive_path = staging.path().join("package");
    let extracted = staging.path().join("extract");

    fetch(&https_client()?, package.archive, &archive_path)?;
    extract(&archive_path, package.archive.format, &extracted)?;

    if let Ok(entries) = fs::read_dir(&extracted) {
        for directory in entries.filter_map(Result::ok).map(|entry| entry.path()) {
            let client = binary_path(&directory, CLIENT);
            if client.is_file() {
                ensure_executable(&client).map_err(install_failed)?;
            }
        }
    }
    let release = find_release(&extracted).ok_or_else(|| {
        ScrcpyProblem::new(
            ScrcpyDiagnosticCode::ArchiveInvalid,
            "the package has no scrcpy and scrcpy-server",
        )
    })?;

    let backup = staging.path().join(PREVIOUS);
    swap_directory(&release, install_root, &backup).map_err(install_failed)?;

    Ok(install_root.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_release_directory_inside_the_package() {
        let folder = tempfile::tempdir().unwrap();
        let release = folder.path().join("scrcpy-linux-x86_64-v5.0");
        fs::create_dir_all(&release).unwrap();
        let client = binary_path(&release, CLIENT);
        fs::write(&client, "#!/bin/sh\n").unwrap();
        ensure_executable(&client).unwrap();

        assert_eq!(find_release(folder.path()), None);

        fs::write(release.join(super::super::resolve::SERVER), b"jar").unwrap();
        assert_eq!(find_release(folder.path()), Some(release));
    }

    #[test]
    fn puts_the_previous_release_back_after_an_interrupted_swap() {
        let folder = tempfile::tempdir().unwrap();
        let install_root = folder.path().join("scrcpy-v3");
        let previous = folder.path().join(".scrcpy-abc").join(PREVIOUS);
        fs::create_dir_all(&previous).unwrap();
        let client = binary_path(&previous, CLIENT);
        fs::write(&client, "#!/bin/sh\n").unwrap();
        ensure_executable(&client).unwrap();
        fs::write(previous.join(super::super::resolve::SERVER), b"jar").unwrap();

        settle_staging(folder.path(), &install_root);

        assert!(release_in(&install_root).is_some());
        assert!(!folder.path().join(".scrcpy-abc").exists());
    }
}
