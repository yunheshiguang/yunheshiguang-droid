use std::sync::Arc;

use tauri::{App, Manager, Runtime};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

use super::identity::DeviceIdentities;
use super::service::AdbService;

pub struct AdbModule;

impl<R: Runtime> WorkspaceModule<R> for AdbModule {
    fn id(&self) -> &'static str {
        "adb"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::new(AdbService::default()));
        app.manage(Arc::new(DeviceIdentities::default()));

        Ok(())
    }
}
