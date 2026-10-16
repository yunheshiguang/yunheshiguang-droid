use crate::workspace::adb::{AdbCommand, TRANSFER_POLICY, required_file};
use crate::workspace::contract::*;
use crate::workspace::messages::*;

use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::ONLINE;
use super::command::AdbCommandAction;

fn install(request: &BatchActionRequest) -> Result<AdbCommand, String> {
    let apk = required_file(&request.path, "APK")?;
    Ok(AdbCommand::new(
        ["install".into(), "-r".into(), apk.into_os_string()],
        TRANSFER_POLICY,
    ))
}

fn upload(request: &BatchActionRequest) -> Result<AdbCommand, String> {
    let local = required_file(&request.path, "upload file")?;
    let file_name = local
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "upload file name is not valid UTF-8".to_string())?;
    let remote = format!("/sdcard/Download/{file_name}");

    Ok(AdbCommand::new(
        ["push".into(), local.into_os_string(), remote.into()],
        TRANSFER_POLICY,
    ))
}

pub(super) static INSTALL_APK: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::InstallApk,
        slug: "install-apk",
        title_key: TASK_TITLE_INSTALL_APK_KEY,
        requirement: ActionRequirement::SourceFile,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: install,
};

pub(super) static UPLOAD_FILE: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::UploadFile,
        slug: "upload-file",
        title_key: TASK_TITLE_UPLOAD_FILE_KEY,
        requirement: ActionRequirement::SourceFile,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: upload,
};
