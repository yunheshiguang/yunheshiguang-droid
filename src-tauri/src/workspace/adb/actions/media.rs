use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use tempfile::NamedTempFile;

use crate::workspace::adb::{AdbClient, AdbCommandError, CommandPolicy};

use super::artifact::{
    output_path, persist_new, prepare_output_dir, unique_suffix, validate_mp4, validate_png,
};

pub fn screenshot(
    client: &AdbClient,
    serial: &str,
    output_dir: &Path,
) -> Result<(), AdbCommandError> {
    prepare_output_dir(output_dir)?;
    let temp = NamedTempFile::new_in(output_dir).map_err(AdbCommandError::from_io)?;
    let output_file = temp
        .as_file()
        .try_clone()
        .map_err(AdbCommandError::from_io)?;
    client.run_serial_to_file(
        serial,
        [
            OsString::from("exec-out"),
            OsString::from("screencap"),
            OsString::from("-p"),
        ],
        CommandPolicy::new(Duration::from_secs(60), 64 * 1024 * 1024),
        output_file,
    )?;
    temp.as_file()
        .sync_all()
        .map_err(AdbCommandError::from_io)?;
    validate_png(temp.path())?;
    let target = output_path(output_dir, "screenshot", serial, "png");
    persist_new(temp, &target)
}

pub fn screen_record(
    client: &AdbClient,
    serial: &str,
    output_dir: &Path,
) -> Result<(), AdbCommandError> {
    prepare_output_dir(output_dir)?;
    let suffix = unique_suffix();
    let remote = format!("/sdcard/yunheshiguang-record-{suffix}.mp4");
    let recording = client.run_serial_with_policy(
        serial,
        [
            OsString::from("shell"),
            OsString::from("screenrecord"),
            OsString::from("--time-limit"),
            OsString::from("30"),
            OsString::from(&remote),
        ],
        CommandPolicy::new(Duration::from_secs(40), 1024 * 1024),
    );

    let result = recording.and_then(|_| {
        let target = output_path(output_dir, "recording", serial, "mp4");
        pull_to_new_file(client, serial, &remote, output_dir, &target)
    });
    cleanup_remote_file(client, serial, &remote);
    result
}

fn cleanup_remote_file(client: &AdbClient, serial: &str, remote: &str) {
    let _ = client.run_serial(
        serial,
        [
            OsString::from("shell"),
            OsString::from("rm"),
            OsString::from("-f"),
            OsString::from(remote),
        ],
    );
}

fn pull_to_new_file(
    client: &AdbClient,
    serial: &str,
    remote: &str,
    output_dir: &Path,
    target: &Path,
) -> Result<(), AdbCommandError> {
    let temp = NamedTempFile::new_in(output_dir).map_err(AdbCommandError::from_io)?;
    client.run_serial_with_policy(
        serial,
        [
            OsString::from("pull"),
            OsString::from(remote),
            temp.path().as_os_str().to_os_string(),
        ],
        CommandPolicy::new(Duration::from_secs(120), 4 * 1024 * 1024),
    )?;
    temp.as_file()
        .sync_all()
        .map_err(AdbCommandError::from_io)?;
    validate_mp4(temp.path())?;
    persist_new(temp, target)
}
