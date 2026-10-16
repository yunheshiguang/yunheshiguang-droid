use crate::workspace::adb::{AdbCommand, AdbDeviceState};
use crate::workspace::contract::*;
use crate::workspace::messages::*;

use super::super::executor::each_device;
use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::super::task::runtime_error_detail;
use super::{ActionContext, ActionRun, BatchActionHandler, ONLINE, PreparedAction};

pub(super) struct AdbCommandAction {
    pub spec: BatchActionSpec,
    pub allowed_states: &'static [AdbDeviceState],
    pub command: fn(&BatchActionRequest) -> Result<AdbCommand, String>,
}

impl BatchActionHandler for AdbCommandAction {
    fn spec(&self) -> &BatchActionSpec {
        &self.spec
    }

    fn allowed_states(&self) -> &'static [AdbDeviceState] {
        self.allowed_states
    }

    fn prepare(
        &self,
        request: &BatchActionRequest,
        _context: &ActionContext<'_>,
    ) -> Result<Box<dyn PreparedAction>, UiText> {
        match (self.command)(request) {
            Ok(command) => Ok(Box::new(command)),
            Err(reason) => Err(runtime_error_detail(reason)),
        }
    }
}

impl PreparedAction for AdbCommand {
    fn run(&self, context: &ActionContext<'_>, serials: Vec<String>) -> ActionRun {
        ActionRun {
            device_results: each_device(context.adb_path, serials, |client, serial| {
                AdbCommand::run(self, client, serial)
            }),
            ..ActionRun::default()
        }
    }
}

fn unspecified(_request: &BatchActionRequest) -> Result<AdbCommand, String> {
    Err("batch action is unspecified".to_string())
}

pub(super) static UNSPECIFIED: AdbCommandAction = AdbCommandAction {
    spec: BatchActionSpec {
        kind: BatchActionKind::Unspecified,
        slug: "batch",
        title_key: TASK_TITLE_DEFAULT_KEY,
        requirement: ActionRequirement::Nothing,
        success_detail: SuccessDetail::DeviceCount,
    },
    allowed_states: ONLINE,
    command: unspecified,
};
