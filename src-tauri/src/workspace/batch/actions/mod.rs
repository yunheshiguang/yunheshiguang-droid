mod apps;
mod capture;
mod command;
mod connection;
mod input;
mod script;
mod transfer;

use std::path::Path;

use yunheshiguang_sdk::documents::DocumentStore;

use crate::workspace::adb::{AdbDeviceState, DeviceIdentities};
use crate::workspace::contract::*;
use crate::workspace::input::InputRouter;
use crate::workspace::scripts::ScriptRunner;

use super::spec::BatchActionSpec;

pub(super) struct ActionContext<'a> {
    pub adb_path: &'a Path,
    pub identities: &'a DeviceIdentities,
    pub inputs: &'a InputRouter,
    pub scripts: &'a ScriptRunner,
    pub documents: &'a dyn DocumentStore,
}

#[derive(Default)]
pub(super) struct ActionRun {
    pub device_results: Vec<AdbDeviceActionResult>,
    pub input_routes: Vec<DeviceInputRoute>,
    pub script_outputs: Vec<DeviceScriptOutput>,
}

pub(super) const ONLINE: &[AdbDeviceState] = &[AdbDeviceState::Online];

pub(super) trait BatchActionHandler: Sync {
    fn spec(&self) -> &BatchActionSpec;

    fn allowed_states(&self) -> &'static [AdbDeviceState] {
        ONLINE
    }

    fn prepare(
        &self,
        request: &BatchActionRequest,
        context: &ActionContext<'_>,
    ) -> Result<Box<dyn PreparedAction>, UiText>;
}

pub(super) trait PreparedAction {
    fn run(&self, context: &ActionContext<'_>, serials: Vec<String>) -> ActionRun;
}

pub(super) fn handler(kind: BatchActionKind) -> &'static dyn BatchActionHandler {
    match kind {
        BatchActionKind::Screenshot => &capture::SCREENSHOT,
        BatchActionKind::ScreenRecord => &capture::SCREEN_RECORD,
        BatchActionKind::InstallApk => &transfer::INSTALL_APK,
        BatchActionKind::UploadFile => &transfer::UPLOAD_FILE,
        BatchActionKind::InputText => &input::INPUT_TEXT,
        BatchActionKind::ClipboardWrite => &input::CLIPBOARD_WRITE,
        BatchActionKind::LaunchApp => &apps::LAUNCH_APP,
        BatchActionKind::StopApp => &apps::STOP_APP,
        BatchActionKind::ClearAppData => &apps::CLEAR_APP_DATA,
        BatchActionKind::UninstallApp => &apps::UNINSTALL_APP,
        BatchActionKind::RecoverConnection => &connection::RECOVER_CONNECTION,
        BatchActionKind::RunScript => &script::RUN_SCRIPT,
        BatchActionKind::Unspecified => &command::UNSPECIFIED,
    }
}

#[cfg(test)]
mod tests;
