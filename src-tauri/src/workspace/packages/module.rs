use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::documents::DocumentStore;

use crate::workspace::events::WorkspaceEvents;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::service::PackagesService;
use super::signing::SigningKeys;

const PACKAGES_DIRECTORY: &str = "packages";

pub struct PackagesModule;

impl<R: Runtime> WorkspaceModule<R> for PackagesModule {
    fn id(&self) -> &'static str {
        "packages"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let directory = app.path().app_local_data_dir()?.join(PACKAGES_DIRECTORY);

        app.manage(Arc::new(PackagesService {
            documents: dependency::<dyn DocumentStore, R>(app)?,
            catalog: dependency(app)?,
            governance: dependency(app)?,
            telemetry: dependency(app)?,
            events: dependency::<dyn WorkspaceEvents, R>(app)?,
            keys: Arc::new(SigningKeys::new(directory)),
        }));

        let rpc = super::rpc::PackagesRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
