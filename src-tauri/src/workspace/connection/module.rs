use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::connection::ConnectionProvider;

use crate::workspace::adb::AdbService;
use crate::workspace::android_tools::AndroidPlatformToolsRegistry;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::adb_provider::AdbConnectionProvider;

pub struct ConnectionModule {
    contributed: Option<Arc<dyn ConnectionProvider>>,
}

impl ConnectionModule {
    pub fn new(contributed: Option<Arc<dyn ConnectionProvider>>) -> Self {
        Self { contributed }
    }
}

fn adb_provider<R: Runtime>(app: &App<R>) -> Result<Arc<dyn ConnectionProvider>, ModuleSetupError> {
    let registry = app
        .try_state::<Arc<AndroidPlatformToolsRegistry>>()
        .ok_or("the android tools module has to be set up first")?;
    let adb = app
        .try_state::<Arc<AdbService>>()
        .ok_or("the adb module has to be set up first")?;

    Ok(Arc::new(AdbConnectionProvider::new(
        Arc::clone(registry.inner()),
        Arc::clone(adb.inner()),
    )))
}

impl<R: Runtime> WorkspaceModule<R> for ConnectionModule {
    fn id(&self) -> &'static str {
        "connection"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let provider = match &self.contributed {
            Some(provider) => Arc::clone(provider),
            None => adb_provider(app)?,
        };

        app.manage(provider);

        let rpc = super::ConnectionRpc {
            governance: dependency(app)?,
            provider: dependency::<dyn yunheshiguang_sdk::connection::ConnectionProvider, R>(app)?,
            telemetry: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
