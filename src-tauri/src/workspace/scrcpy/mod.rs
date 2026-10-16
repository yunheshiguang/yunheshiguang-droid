mod args;
mod diagnostics;
mod install;
mod package;
mod resolve;
mod rpc;
mod service;
mod sessions;

use std::sync::Arc;

use tauri::{App, Manager, Runtime};

use crate::workspace::lifecycle::Shutdown;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

pub use service::ScrcpyService;

const SCRCPY_DIRECTORY: &str = "scrcpy";

pub struct ScrcpyModule;

impl<R: Runtime> WorkspaceModule<R> for ScrcpyModule {
    fn id(&self) -> &'static str {
        "scrcpy"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let directory = app.path().app_local_data_dir()?.join(SCRCPY_DIRECTORY);
        let service = ScrcpyService::new(
            directory,
            dependency(app)?,
            dependency(app)?,
            dependency(app)?,
            dependency(app)?,
        );

        let service = Arc::new(service);
        let stopping = Arc::clone(&service);
        dependency::<Shutdown, R>(app)?.on_exit(move || stopping.stop_all());
        app.manage(service);

        let rpc = rpc::ScrcpyRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
