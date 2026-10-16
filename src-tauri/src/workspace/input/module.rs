use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::input::InputChannel;

use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::router::InputRouter;
use super::yadb::{YadbChannel, install_host_copy};

const CACHE_DIRECTORY: &str = "input";

pub struct InputModule {
    contributed: Mutex<Vec<Box<dyn InputChannel>>>,
}

impl InputModule {
    pub fn new(contributed: Vec<Box<dyn InputChannel>>) -> Self {
        Self {
            contributed: Mutex::new(contributed),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for InputModule {
    fn id(&self) -> &'static str {
        "input"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let contributed = std::mem::take(
            &mut *self
                .contributed
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        let core: Vec<Box<dyn InputChannel>> =
            match install_host_copy(&app.path().app_cache_dir()?.join(CACHE_DIRECTORY)) {
                Ok(host_copy) => vec![Box::new(YadbChannel::new(host_copy))],
                Err(error) => {
                    log::error!("input: YADB could not be prepared: {error}");
                    Vec::new()
                }
            };

        app.manage(Arc::new(InputRouter::new(contributed, core)));

        let rpc = super::rpc::InputRpc(dependency(app)?);
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
