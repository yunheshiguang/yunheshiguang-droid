use crate::workspace::adb::{ACTION_POLICY, AdbCommand, AdbDeviceState};
use crate::workspace::contract::*;
use crate::workspace::messages::*;

use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::command::AdbCommandAction;

fn reconnect(_request: &BatchActionRequest) -> Result<AdbCommand, String> {
    Ok(AdbCommand::new(["reconnect"], ACTION_POLICY))
}

pub(super) static RECOVER_CONNECTION: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::RecoverConnection,
        slug: "recover-connection",
        title_key: TASK_TITLE_RECOVER_CONNECTION_KEY,
        requirement: ActionRequirement::Nothing,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: &[AdbDeviceState::Online, AdbDeviceState::Offline],
    command: reconnect,
};
