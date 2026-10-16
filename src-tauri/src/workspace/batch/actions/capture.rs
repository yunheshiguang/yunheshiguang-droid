use std::path::{Path, PathBuf};

use crate::workspace::adb::{AdbClient, AdbCommandError, required_path, screen_record, screenshot};
use crate::workspace::contract::*;
use crate::workspace::messages::*;

use super::super::executor::each_device;
use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::super::task::runtime_error_detail;
use super::{ActionContext, ActionRun, BatchActionHandler, PreparedAction};

type CaptureFn = fn(&AdbClient, &str, &Path) -> Result<(), AdbCommandError>;

pub(super) struct Capture {
    spec: BatchActionSpec,
    field: &'static str,
    capture: CaptureFn,
}

struct PreparedCapture {
    output_dir: PathBuf,
    capture: CaptureFn,
}

impl BatchActionHandler for Capture {
    fn spec(&self) -> &BatchActionSpec {
        &self.spec
    }

    fn prepare(
        &self,
        request: &BatchActionRequest,
        _context: &ActionContext<'_>,
    ) -> Result<Box<dyn PreparedAction>, UiText> {
        let output_dir = required_path(&request.path, self.field).map_err(runtime_error_detail)?;

        Ok(Box::new(PreparedCapture {
            output_dir,
            capture: self.capture,
        }))
    }
}

impl PreparedAction for PreparedCapture {
    fn run(&self, context: &ActionContext<'_>, serials: Vec<String>) -> ActionRun {
        ActionRun {
            device_results: each_device(context.adb_path, serials, |client, serial| {
                (self.capture)(client, serial, &self.output_dir)
            }),
            ..ActionRun::default()
        }
    }
}

pub(super) static SCREENSHOT: Capture = Capture {
    spec: BatchActionSpec {
        kind: BatchActionKind::Screenshot,
        slug: "screenshot",
        title_key: TASK_TITLE_SCREENSHOT_KEY,
        requirement: ActionRequirement::SavePath,
        success_detail: SuccessDetail::DeviceCount,
    },
    field: "screenshot output directory",
    capture: screenshot,
};

pub(super) static SCREEN_RECORD: Capture = Capture {
    spec: BatchActionSpec {
        kind: BatchActionKind::ScreenRecord,
        slug: "screen-record",
        title_key: TASK_TITLE_SCREEN_RECORD_KEY,
        requirement: ActionRequirement::SavePath,
        success_detail: SuccessDetail::DeviceCountAndPath,
    },
    field: "screen recording output directory",
    capture: screen_record,
};
