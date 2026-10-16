use crate::workspace::adb::validate_script_id;
use crate::workspace::contract::*;
use crate::workspace::messages::*;
use crate::workspace::scripts::load_script;

use super::super::executor::run_scripts_concurrently;
use super::super::spec::{ActionRequirement, BatchActionSpec, SuccessDetail};
use super::super::task::runtime_error_detail;
use super::{ActionContext, ActionRun, BatchActionHandler, PreparedAction};

pub(super) struct RunScript {
    spec: BatchActionSpec,
}

struct PreparedScript {
    script_id: String,
    script: Script,
}

impl BatchActionHandler for RunScript {
    fn spec(&self) -> &BatchActionSpec {
        &self.spec
    }

    fn prepare(
        &self,
        request: &BatchActionRequest,
        context: &ActionContext<'_>,
    ) -> Result<Box<dyn PreparedAction>, UiText> {
        let script_id = validate_script_id(&request.script_id).map_err(runtime_error_detail)?;
        let script = load_script(context.documents, &script_id)?;

        Ok(Box::new(PreparedScript { script_id, script }))
    }
}

impl PreparedAction for PreparedScript {
    fn run(&self, context: &ActionContext<'_>, serials: Vec<String>) -> ActionRun {
        let identities = context.identities.resolve(context.adb_path, &serials);
        let (device_results, script_outputs) = run_scripts_concurrently(
            context.adb_path,
            serials,
            &self.script_id,
            &self.script,
            context.scripts,
            &identities,
        );

        ActionRun {
            device_results,
            script_outputs,
            ..ActionRun::default()
        }
    }
}

pub(super) static RUN_SCRIPT: RunScript = RunScript {
    spec: BatchActionSpec {
        kind: BatchActionKind::RunScript,
        slug: "run-script",
        title_key: TASK_TITLE_RUN_SCRIPT_KEY,
        requirement: ActionRequirement::Script,
        success_detail: SuccessDetail::DeviceCount,
    },
};
