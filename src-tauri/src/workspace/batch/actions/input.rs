use yunheshiguang_sdk::input::InputRequest;

use crate::workspace::adb::validate_input_text;
use crate::workspace::contract::*;
use crate::workspace::messages::*;

use super::super::executor::deliver_concurrently;
use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::super::task::runtime_error_detail;
use super::{ActionContext, ActionRun, BatchActionHandler, PreparedAction};

type RequestFn = for<'a> fn(&'a str) -> InputRequest<'a>;

fn text(value: &str) -> InputRequest<'_> {
    InputRequest::Text(value)
}

fn clipboard(value: &str) -> InputRequest<'_> {
    InputRequest::Clipboard(value)
}

pub(super) struct Input {
    spec: BatchActionSpec,
    field: &'static str,
    request: RequestFn,
}

struct PreparedInput {
    text: String,
    request: RequestFn,
}

impl BatchActionHandler for Input {
    fn spec(&self) -> &BatchActionSpec {
        &self.spec
    }

    fn prepare(
        &self,
        request: &BatchActionRequest,
        _context: &ActionContext<'_>,
    ) -> Result<Box<dyn PreparedAction>, UiText> {
        let text = validate_input_text(&request.text, self.field).map_err(runtime_error_detail)?;

        Ok(Box::new(PreparedInput {
            text,
            request: self.request,
        }))
    }
}

impl PreparedAction for PreparedInput {
    fn run(&self, context: &ActionContext<'_>, serials: Vec<String>) -> ActionRun {
        let identities = context.identities.resolve(context.adb_path, &serials);
        let (device_results, input_routes) = deliver_concurrently(
            context.adb_path,
            serials,
            (self.request)(&self.text),
            context.inputs,
            &identities,
        );

        ActionRun {
            device_results,
            input_routes,
            ..ActionRun::default()
        }
    }
}

pub(super) static INPUT_TEXT: Input = Input {
    spec: BatchActionSpec {
        kind: BatchActionKind::InputText,
        slug: "input-text",
        title_key: TASK_TITLE_INPUT_TEXT_KEY,
        requirement: ActionRequirement::Text,
        success_detail: SuccessDetail::DeviceCount,
    },
    field: "input text",
    request: text,
};

pub(super) static CLIPBOARD_WRITE: Input = Input {
    spec: BatchActionSpec {
        kind: BatchActionKind::ClipboardWrite,
        slug: "clipboard-write",
        title_key: TASK_TITLE_CLIPBOARD_WRITE_KEY,
        requirement: ActionRequirement::Text,
        success_detail: SuccessDetail::DeviceCount,
    },
    field: "clipboard text",
    request: clipboard,
};
