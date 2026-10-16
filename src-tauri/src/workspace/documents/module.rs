use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::documents::{DocumentStore, FolderDocumentStore};

use crate::workspace::events::WorkspaceEvents;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::service::DocumentsService;

const DOCUMENTS_DIRECTORY: &str = "documents";

pub struct DocumentsModule {
    contributed: Option<Arc<dyn DocumentStore>>,
}

impl DocumentsModule {
    pub fn new(contributed: Option<Arc<dyn DocumentStore>>) -> Self {
        Self { contributed }
    }
}

impl<R: Runtime> WorkspaceModule<R> for DocumentsModule {
    fn id(&self) -> &'static str {
        "documents"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let store: Arc<dyn DocumentStore> = match &self.contributed {
            Some(store) => Arc::clone(store),
            None => Arc::new(FolderDocumentStore::new(
                app.path().app_data_dir()?.join(DOCUMENTS_DIRECTORY),
            )),
        };
        let service = Arc::new(DocumentsService::new(
            Arc::clone(&store),
            dependency(app)?,
            dependency::<dyn WorkspaceEvents, R>(app)?,
        ));

        app.manage(store);
        app.manage(service);

        let rpc = super::rpc::DocumentsRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
