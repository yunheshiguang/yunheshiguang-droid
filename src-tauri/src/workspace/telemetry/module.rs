use std::sync::Arc;

use tauri::{App, Manager, Runtime};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

use super::hub::Telemetry;

pub struct TelemetryModule {
    telemetry: Arc<Telemetry>,
}

impl TelemetryModule {
    pub fn new(telemetry: Telemetry) -> Self {
        Self {
            telemetry: Arc::new(telemetry),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for TelemetryModule {
    fn id(&self) -> &'static str {
        "telemetry"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::clone(&self.telemetry));
        let rpc = super::rpc::TelemetryRpc(Arc::clone(&self.telemetry));
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
