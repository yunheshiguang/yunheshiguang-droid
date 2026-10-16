use std::sync::Arc;

use tauri::{App, Manager, Runtime};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::runtime_config::AndroidPlatformToolsRegistry;

pub struct AndroidToolsModule;

impl<R: Runtime> WorkspaceModule<R> for AndroidToolsModule {
    fn id(&self) -> &'static str {
        "android-platform-tools"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let registry = AndroidPlatformToolsRegistry::from_app(app.handle())?;
        app.manage(Arc::new(registry));

        let rpc = super::PlatformToolsRpc {
            registry: dependency(app)?,
            adb: dependency(app)?,
            governance: dependency(app)?,
            telemetry: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
