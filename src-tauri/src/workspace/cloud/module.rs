use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::cloud::CloudPhoneProvider;
use yunheshiguang_sdk::documents::DocumentStore;

use crate::workspace::documents::DocumentsService;
use crate::workspace::events::WorkspaceEvents;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::registered::{CLOUD_PHONES_COLLECTION, RegisteredPhones};
use super::service::CloudService;

const REFRESH_EVERY: Duration = Duration::from_secs(5 * 60);

pub struct CloudModule {
    contributed: Mutex<Vec<Box<dyn CloudPhoneProvider>>>,
}

impl CloudModule {
    pub fn new(contributed: Vec<Box<dyn CloudPhoneProvider>>) -> Self {
        Self {
            contributed: Mutex::new(contributed),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for CloudModule {
    fn id(&self) -> &'static str {
        "cloud"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        dependency::<DocumentsService, R>(app)?.reserve(CLOUD_PHONES_COLLECTION);
        let registered = Arc::new(RegisteredPhones::new(dependency::<dyn DocumentStore, R>(
            app,
        )?));
        let mut providers = std::mem::take(
            &mut *self
                .contributed
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        )
        .into_iter()
        .map(Arc::from)
        .collect::<Vec<Arc<dyn CloudPhoneProvider>>>();
        providers.push(Arc::clone(&registered) as Arc<dyn CloudPhoneProvider>);

        let service = Arc::new(CloudService {
            providers,
            registered,
            governance: dependency(app)?,
            telemetry: dependency(app)?,
            events: dependency::<dyn WorkspaceEvents, R>(app)?,
            cache: Mutex::new(Vec::new()),
        });
        let refresher = Arc::clone(&service);
        tauri::async_runtime::spawn(async move {
            loop {
                refresher.list().await;
                tokio::time::sleep(REFRESH_EVERY).await;
            }
        });
        app.manage(service);

        let rpc = super::rpc::CloudRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
