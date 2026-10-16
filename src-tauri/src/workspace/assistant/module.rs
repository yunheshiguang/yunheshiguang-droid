use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::assistant::AssistantProvider;
use yunheshiguang_sdk::documents::DocumentStore;

use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::openai::OpenAiCompatible;
use super::service::AssistantService;
use super::settings::SettingsStore;

const ASSISTANT_DIRECTORY: &str = "assistant";

pub struct AssistantModule {
    providers: Mutex<Vec<Box<dyn AssistantProvider>>>,
}

impl AssistantModule {
    pub fn new(providers: Vec<Box<dyn AssistantProvider>>) -> Self {
        Self {
            providers: Mutex::new(providers),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for AssistantModule {
    fn id(&self) -> &'static str {
        "assistant"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let settings = Arc::new(SettingsStore::load(
            app.path().app_local_data_dir()?.join(ASSISTANT_DIRECTORY),
        ));
        let mut providers = std::mem::take(
            &mut *self
                .providers
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        providers.push(Box::new(OpenAiCompatible::new(Arc::clone(&settings))));

        app.manage(Arc::new(AssistantService {
            providers,
            settings,
            documents: dependency::<dyn DocumentStore, R>(app)?,
            governance: dependency(app)?,
            telemetry: dependency(app)?,
        }));

        let rpc = super::rpc::AssistantRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
