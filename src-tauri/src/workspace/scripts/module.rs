use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::scripts::ScriptEngine;

use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::engine::ShellEngine;
use super::runner::ScriptRunner;

pub struct ScriptsModule {
    contributed: Mutex<Vec<Box<dyn ScriptEngine>>>,
}

impl ScriptsModule {
    pub fn new(contributed: Vec<Box<dyn ScriptEngine>>) -> Self {
        Self {
            contributed: Mutex::new(contributed),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for ScriptsModule {
    fn id(&self) -> &'static str {
        "scripts"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let contributed = std::mem::take(
            &mut *self
                .contributed
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );

        app.manage(Arc::new(ScriptRunner::new(
            contributed,
            vec![Box::new(ShellEngine)],
        )));

        let rpc = super::rpc::ScriptsRpc(dependency(app)?);
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
