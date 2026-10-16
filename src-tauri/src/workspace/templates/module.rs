use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::templates::TemplateSource;

use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::catalog::TemplateCatalog;

pub struct TemplatesModule {
    catalog: Arc<TemplateCatalog>,
}

impl TemplatesModule {
    pub fn new(contributed: Vec<Box<dyn TemplateSource>>) -> Self {
        Self {
            catalog: Arc::new(TemplateCatalog::new(contributed)),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for TemplatesModule {
    fn id(&self) -> &'static str {
        "templates"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::clone(&self.catalog));

        let rpc = super::rpc::TemplatesRpc {
            catalog: Arc::clone(&self.catalog),
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
