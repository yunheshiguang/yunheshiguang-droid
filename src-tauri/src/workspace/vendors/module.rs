use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::vendor::VendorAdapter;

use crate::workspace::adb::DeviceIdentities;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::registry::VendorRegistry;

pub struct VendorsModule {
    contributed: Mutex<Vec<Box<dyn VendorAdapter>>>,
}

impl VendorsModule {
    pub fn new(contributed: Vec<Box<dyn VendorAdapter>>) -> Self {
        Self {
            contributed: Mutex::new(contributed),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for VendorsModule {
    fn id(&self) -> &'static str {
        "vendors"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let identities = app
            .try_state::<Arc<DeviceIdentities>>()
            .ok_or("the adb module has to be set up first")?;
        let contributed = std::mem::take(
            &mut *self
                .contributed
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );

        app.manage(Arc::new(VendorRegistry::new(
            contributed,
            Arc::clone(identities.inner()),
        )));

        let rpc = super::rpc::VendorsRpc {
            vendors: dependency(app)?,
            registry: dependency(app)?,
            adb: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
