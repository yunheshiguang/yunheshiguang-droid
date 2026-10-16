use crate::workspace::adb::{ACTION_POLICY, AdbCommand, TRANSFER_POLICY, validate_package_name};
use crate::workspace::contract::*;
use crate::workspace::messages::*;

use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::ONLINE;
use super::command::AdbCommandAction;

const PM_SUCCESS: &str = "Success";

fn package(request: &BatchActionRequest) -> Result<String, String> {
    validate_package_name(&request.package_name)
}

fn launch(request: &BatchActionRequest) -> Result<AdbCommand, String> {
    let package = package(request)?;
    Ok(AdbCommand::new(
        ["shell", "monkey", "-p", package.as_str(), "1"],
        ACTION_POLICY,
    ))
}

fn stop(request: &BatchActionRequest) -> Result<AdbCommand, String> {
    let package = package(request)?;
    Ok(AdbCommand::new(
        ["shell", "am", "force-stop", package.as_str()],
        ACTION_POLICY,
    ))
}

fn clear_data(request: &BatchActionRequest) -> Result<AdbCommand, String> {
    let package = package(request)?;
    Ok(
        AdbCommand::new(["shell", "pm", "clear", package.as_str()], ACTION_POLICY)
            .expecting(PM_SUCCESS),
    )
}

fn uninstall(request: &BatchActionRequest) -> Result<AdbCommand, String> {
    let package = package(request)?;
    Ok(AdbCommand::new(["uninstall", package.as_str()], TRANSFER_POLICY).expecting(PM_SUCCESS))
}

pub(super) static LAUNCH_APP: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::LaunchApp,
        slug: "launch-app",
        title_key: TASK_TITLE_LAUNCH_APP_KEY,
        requirement: ActionRequirement::PackageName,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: launch,
};

pub(super) static STOP_APP: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::StopApp,
        slug: "stop-app",
        title_key: TASK_TITLE_STOP_APP_KEY,
        requirement: ActionRequirement::PackageName,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: stop,
};

pub(super) static CLEAR_APP_DATA: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::ClearAppData,
        slug: "clear-app-data",
        title_key: TASK_TITLE_CLEAR_APP_DATA_KEY,
        requirement: ActionRequirement::PackageName,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: clear_data,
};

pub(super) static UNINSTALL_APP: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::UninstallApp,
        slug: "uninstall-app",
        title_key: TASK_TITLE_UNINSTALL_APP_KEY,
        requirement: ActionRequirement::PackageName,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: uninstall,
};
