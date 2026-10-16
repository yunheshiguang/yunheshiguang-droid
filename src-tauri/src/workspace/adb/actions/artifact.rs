use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::NamedTempFile;

use crate::workspace::adb::{AdbCommandError, AdbErrorKind};

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn persist_new(temp: NamedTempFile, target: &Path) -> Result<(), AdbCommandError> {
    temp.persist_noclobber(target)
        .map(|_| ())
        .map_err(|error| AdbCommandError::from_io(error.error))
}

pub(super) fn prepare_output_dir(path: &Path) -> Result<(), AdbCommandError> {
    fs::create_dir_all(path).map_err(AdbCommandError::from_io)?;
    if path.is_dir() {
        Ok(())
    } else {
        Err(AdbCommandError::new(
            AdbErrorKind::Io,
            "output path is not a directory",
        ))
    }
}

pub(super) fn output_path(
    output_dir: &Path,
    prefix: &str,
    serial: &str,
    extension: &str,
) -> PathBuf {
    let serial = serial
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    output_dir.join(format!(
        "{prefix}-{serial}-{}.{}",
        unique_suffix(),
        extension
    ))
}

pub(super) fn unique_suffix() -> u128 {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let sequence = OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    timestamp
        .saturating_mul(1_000_000)
        .saturating_add(sequence as u128)
}

struct FileFormat {
    magic: &'static [u8],
    offset: usize,
    incomplete: &'static str,
    mismatch: &'static str,
}

const PNG: FileFormat = FileFormat {
    magic: &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
    offset: 0,
    incomplete: "ADB screenshot is incomplete",
    mismatch: "ADB screenshot is not a PNG image",
};

const MP4: FileFormat = FileFormat {
    magic: b"ftyp",
    offset: 4,
    incomplete: "ADB recording is incomplete",
    mismatch: "ADB recording is not an MP4 file",
};

fn validate_format(path: &Path, format: &FileFormat) -> Result<(), AdbCommandError> {
    let mut header = vec![0_u8; format.offset + format.magic.len()];

    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|error| {
            AdbCommandError::new(
                AdbErrorKind::InvalidOutput,
                format!("{}: {error}", format.incomplete),
            )
        })?;

    if &header[format.offset..] == format.magic {
        Ok(())
    } else {
        Err(AdbCommandError::new(
            AdbErrorKind::InvalidOutput,
            format.mismatch,
        ))
    }
}

pub(super) fn validate_png(path: &Path) -> Result<(), AdbCommandError> {
    validate_format(path, &PNG)
}

pub(super) fn validate_mp4(path: &Path) -> Result<(), AdbCommandError> {
    validate_format(path, &MP4)
}
