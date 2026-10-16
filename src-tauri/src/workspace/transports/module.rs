use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::transport::TransportAdapter;

use crate::workspace::adb::AdbService;
use crate::workspace::android_tools::AndroidPlatformToolsRegistry;
use crate::workspace::cloud::{CloudService, CloudTransport};
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::builtin::{OtgTransport, UsbTransport, WirelessTransport};
use super::registry::TransportRegistry;

pub struct TransportsModule {
    contributed: Mutex<Vec<Box<dyn TransportAdapter>>>,
}

impl TransportsModule {
    pub fn new(contributed: Vec<Box<dyn TransportAdapter>>) -> Self {
        Self {
            contributed: Mutex::new(contributed),
        }
    }
}

fn core_transports<R: Runtime>(
    app: &App<R>,
) -> Result<Vec<Box<dyn TransportAdapter>>, ModuleSetupError> {
    let registry = app
        .try_state::<Arc<AndroidPlatformToolsRegistry>>()
        .ok_or("the android tools module has to be set up first")?;
    let adb = app
        .try_state::<Arc<AdbService>>()
        .ok_or("the adb module has to be set up first")?;

    Ok(vec![
        Box::new(UsbTransport),
        Box::new(WirelessTransport::new(
            Arc::clone(registry.inner()),
            Arc::clone(adb.inner()),
        )),
        Box::new(OtgTransport),
        Box::new(CloudTransport::new(dependency::<CloudService, R>(app)?)),
    ])
}

impl<R: Runtime> WorkspaceModule<R> for TransportsModule {
    fn id(&self) -> &'static str {
        "transports"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let contributed = std::mem::take(
            &mut *self
                .contributed
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );

        app.manage(Arc::new(TransportRegistry::new(
            core_transports(app)?,
            contributed,
        )));

        let rpc = super::rpc::DiscoveryRpc(dependency(app)?);
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
