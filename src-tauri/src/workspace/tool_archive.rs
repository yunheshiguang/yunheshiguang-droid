use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::time::Duration;

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 768 * 1024 * 1024;
const MAX_ENTRIES: usize = 4096;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(240);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PinnedArchive {
    pub url: &'static str,
    pub size: u64,
    pub sha256: &'static str,
    pub format: ArchiveFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveError {
    Download(String),
    Integrity(String),
    Archive(String),
}

fn archive_error(error: impl ToString) -> ArchiveError {
    ArchiveError::Archive(error.to_string())
}

pub fn https_client() -> Result<Client, ArchiveError> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .https_only(true)
        .build()
        .map_err(|error| ArchiveError::Download(error.to_string()))
}

pub fn fetch(client: &Client, archive: PinnedArchive, path: &Path) -> Result<(), ArchiveError> {
    let download = |error: &dyn ToString| ArchiveError::Download(error.to_string());
    let mut response = client
        .get(archive.url)
        .send()
        .map_err(|error| download(&error))?;

    if !response.status().is_success() {
        return Err(ArchiveError::Download(format!(
            "download failed with status {}",
            response.status()
        )));
    }
    if response.content_length().unwrap_or(0) > MAX_DOWNLOAD_BYTES {
        return Err(ArchiveError::Download(
            "download is larger than the allowed limit".to_string(),
        ));
    }

    let mut file = File::create(path).map_err(|error| download(&error))?;
    let written = io::copy(
        &mut response.by_ref().take(MAX_DOWNLOAD_BYTES + 1),
        &mut file,
    )
    .map_err(|error| download(&error))?;

    if written > MAX_DOWNLOAD_BYTES {
        return Err(ArchiveError::Download(
            "download exceeded the allowed limit".to_string(),
        ));
    }
    if written != archive.size {
        return Err(ArchiveError::Integrity(format!(
            "downloaded package size {written} does not match expected size {}",
            archive.size
        )));
    }

    verify_sha256(path, archive.sha256)
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), ArchiveError> {
    let integrity = |error: io::Error| ArchiveError::Integrity(error.to_string());
    let mut file = File::open(path).map_err(integrity)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(integrity)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    if actual == expected {
        Ok(())
    } else {
        Err(ArchiveError::Integrity(format!(
            "SHA-256 mismatch: expected {expected}, received {actual}"
        )))
    }
}

pub fn extract(path: &Path, format: ArchiveFormat, root: &Path) -> Result<(), ArchiveError> {
    fs::create_dir_all(root).map_err(archive_error)?;

    match format {
        ArchiveFormat::Zip => extract_zip(path, root),
        ArchiveFormat::TarGz => extract_tar_gz(path, root),
    }
}

fn extract_zip(path: &Path, root: &Path) -> Result<(), ArchiveError> {
    let mut archive =
        ZipArchive::new(File::open(path).map_err(archive_error)?).map_err(archive_error)?;

    if archive.len() > MAX_ENTRIES {
        return Err(archive_error("archive contains too many entries"));
    }

    let mut extracted = 0_u64;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(archive_error)?;

        if file.is_symlink() {
            return Err(archive_error("archive contains a symbolic link"));
        }
        let remaining = MAX_EXTRACTED_BYTES.saturating_sub(extracted);
        if file.size() > remaining {
            return Err(archive_error("archive expands beyond the allowed limit"));
        }
        let target = root.join(
            file.enclosed_name()
                .ok_or_else(|| archive_error("archive path is unsafe"))?,
        );

        if file.is_dir() {
            fs::create_dir_all(&target).map_err(archive_error)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(archive_error)?;
        }

        let mut output = File::create(&target).map_err(archive_error)?;
        let written =
            io::copy(&mut file.by_ref().take(remaining + 1), &mut output).map_err(archive_error)?;
        if written > remaining {
            return Err(archive_error("archive expands beyond the allowed limit"));
        }
        extracted += written;

        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(mode))
                .map_err(archive_error)?;
        }
    }

    Ok(())
}

fn extract_tar_gz(path: &Path, root: &Path) -> Result<(), ArchiveError> {
    let decoder = flate2::read::GzDecoder::new(File::open(path).map_err(archive_error)?);
    let mut archive = tar::Archive::new(decoder);
    archive.set_preserve_permissions(true);

    let mut entries = 0_usize;
    let mut extracted = 0_u64;
    for entry in archive.entries().map_err(archive_error)? {
        let mut entry = entry.map_err(archive_error)?;
        let kind = entry.header().entry_type();

        entries += 1;
        if entries > MAX_ENTRIES {
            return Err(archive_error("archive contains too many entries"));
        }
        if !(kind.is_file() || kind.is_dir()) {
            return Err(archive_error("archive contains a link or a special file"));
        }
        extracted = extracted.saturating_add(entry.size());
        if extracted > MAX_EXTRACTED_BYTES {
            return Err(archive_error("archive expands beyond the allowed limit"));
        }
        if !entry.unpack_in(root).map_err(archive_error)? {
            return Err(archive_error("archive path is unsafe"));
        }
    }

    Ok(())
}

pub fn swap_directory(staged: &Path, target: &Path, backup: &Path) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    if target.exists() {
        if backup.exists() {
            fs::remove_dir_all(backup)?;
        }
        fs::rename(target, backup)?;
    }

    fs::rename(staged, target).inspect_err(|_| {
        if backup.exists() {
            let _ = fs::rename(backup, target);
        }
    })
}

#[cfg(unix)]
pub fn ensure_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = fs::metadata(path)?.permissions().mode();
    fs::set_permissions(path, fs::Permissions::from_mode(mode | 0o111))
}

#[cfg(not(unix))]
pub fn ensure_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::*;

    fn tar_gz(entries: impl FnOnce(&mut tar::Builder<GzEncoder<File>>)) -> tempfile::TempPath {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut builder =
            tar::Builder::new(GzEncoder::new(file.reopen().unwrap(), Compression::fast()));
        entries(&mut builder);
        builder.into_inner().unwrap().finish().unwrap();
        file.into_temp_path()
    }

    fn header(path: &str, size: u64, kind: tar::EntryType) -> tar::Header {
        let mut header = tar::Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_size(size);
        header.set_entry_type(kind);
        header.set_mode(0o755);
        header.set_cksum();
        header
    }

    #[test]
    fn unpacks_a_tar_gz_release() {
        let archive = tar_gz(|builder| {
            let header = header("scrcpy-v1/scrcpy", 4, tar::EntryType::Regular);
            builder.append(&header, &b"#!sh"[..]).unwrap();
        });
        let root = tempfile::tempdir().unwrap();

        extract(&archive, ArchiveFormat::TarGz, root.path()).unwrap();

        assert_eq!(
            fs::read(root.path().join("scrcpy-v1/scrcpy")).unwrap(),
            b"#!sh"
        );
    }

    #[test]
    fn refuses_links_in_a_tar_gz() {
        let archive = tar_gz(|builder| {
            let mut header = header("scrcpy", 0, tar::EntryType::Symlink);
            header.set_link_name("/etc/passwd").unwrap();
            header.set_cksum();
            builder.append(&header, io::empty()).unwrap();
        });
        let root = tempfile::tempdir().unwrap();

        assert_eq!(
            extract(&archive, ArchiveFormat::TarGz, root.path()),
            Err(archive_error("archive contains a link or a special file"))
        );
    }

    #[test]
    fn unpacks_a_deflated_zip_release() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        zip.add_directory("platform-tools/", options).unwrap();
        zip.start_file("platform-tools/adb", options).unwrap();
        zip.write_all(b"#!sh").unwrap();
        zip.finish().unwrap();
        let root = tempfile::tempdir().unwrap();

        extract(file.path(), ArchiveFormat::Zip, root.path()).unwrap();

        let executable = root.path().join("platform-tools/adb");
        assert_eq!(fs::read(&executable).unwrap(), b"#!sh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(
                fs::metadata(executable).unwrap().permissions().mode() & 0o777,
                0o755
            );
        }
    }

    #[test]
    fn refuses_a_zip_entry_that_leaves_the_root() {
        for name in [
            "../escaped",
            r"\\?\C:\../escaped",
            r"\\?\UNC\server\share\../escaped",
        ] {
            let file = tempfile::NamedTempFile::new().unwrap();
            let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"x").unwrap();
            zip.finish().unwrap();
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().join("contents");

            assert_eq!(
                extract(file.path(), ArchiveFormat::Zip, &root),
                Err(archive_error("archive path is unsafe")),
                "{name}"
            );
            assert!(!directory.path().join("escaped").exists());
        }
    }

    #[test]
    fn refuses_links_in_a_zip() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
        zip.add_symlink(
            "adb",
            "../escaped",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.finish().unwrap();
        let root = tempfile::tempdir().unwrap();

        assert_eq!(
            extract(file.path(), ArchiveFormat::Zip, root.path()),
            Err(archive_error("archive contains a symbolic link"))
        );
    }

    #[test]
    fn checks_the_digest() {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(file.path(), b"abc").unwrap();

        assert!(
            verify_sha256(
                file.path(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            )
            .is_ok()
        );
        assert!(matches!(
            verify_sha256(file.path(), "00"),
            Err(ArchiveError::Integrity(_))
        ));
    }
}
