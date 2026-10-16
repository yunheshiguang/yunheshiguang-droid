use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::documents::DocumentStore;

use crate::workspace::documents::DocumentsService;
use crate::workspace::governance::Governance;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::book::{GROUPS_COLLECTION, PROFILES_COLLECTION, ProfileBook};
use super::groups::ProfileGroups;
use super::service::ProfilesService;

pub struct ProfilesModule;

impl<R: Runtime> WorkspaceModule<R> for ProfilesModule {
    fn id(&self) -> &'static str {
        "profiles"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let documents = dependency::<DocumentsService, R>(app)?;
        documents.reserve(PROFILES_COLLECTION);
        documents.reserve(GROUPS_COLLECTION);
        let book = Arc::new(ProfileBook::new(dependency::<dyn DocumentStore, R>(app)?));
        let governance = dependency::<Governance, R>(app)?;
        let identities = dependency(app)?;
        governance.install_group_directory(Arc::new(ProfileGroups {
            book: Arc::clone(&book),
            identities: Arc::clone(&identities),
        }));
        let service = ProfilesService {
            book: Arc::clone(&book),
            identities,
            registry: dependency(app)?,
            adb: dependency(app)?,
            governance,
            telemetry: dependency(app)?,
            events: dependency(app)?,
        };

        app.manage(book);
        app.manage(Arc::new(service));

        let rpc = super::rpc::ProfilesRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
