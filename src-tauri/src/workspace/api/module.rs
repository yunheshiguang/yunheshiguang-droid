use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Listener, Manager, Runtime};
use yunheshiguang_sdk::api::ApiAuthenticator;

use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::auth::{Authenticators, CoreTokens};
use super::gateway::ApiGateway;
use super::relay::{EventRelay, FORWARDED_EVENTS};
use super::server::ApiContext;
use super::service::{ApiService, load_config};
use super::tokens::TokenStore;

const API_DIRECTORY: &str = "api";

pub struct ApiModule {
    contributed: Mutex<Vec<Box<dyn ApiAuthenticator>>>,
}

impl ApiModule {
    pub fn new(contributed: Vec<Box<dyn ApiAuthenticator>>) -> Self {
        Self {
            contributed: Mutex::new(contributed),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for ApiModule {
    fn id(&self) -> &'static str {
        "api"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let directory = app.path().app_local_data_dir()?.join(API_DIRECTORY);
        let tokens = Arc::new(TokenStore::load(directory.join("tokens.json")));
        let contributed = std::mem::take(
            &mut *self
                .contributed
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        let authenticators = Arc::new(Authenticators::new(
            contributed,
            vec![Box::new(CoreTokens(Arc::clone(&tokens)))],
        ));
        let relay = Arc::new(EventRelay::default());

        for event in FORWARDED_EVENTS {
            let relay = Arc::clone(&relay);
            app.listen_any(event, move |emitted| {
                relay.publish(event, emitted.payload())
            });
        }

        let context = ApiContext {
            rpc: dependency(app)?,
            authenticators: Arc::clone(&authenticators),
            relay,
        };
        let config_path = directory.join("gateway.json");
        let service = Arc::new(ApiService {
            gateway: ApiGateway::new(context),
            tokens,
            authenticators,
            config: Mutex::new(load_config(&config_path)),
            config_path: Some(config_path),
            governance: dependency(app)?,
            telemetry: dependency(app)?,
        });

        let restoring = Arc::clone(&service);
        app.manage(service);

        let rpc = super::rpc::ApiGatewayRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;
        tauri::async_runtime::spawn(async move { restoring.restore().await });

        Ok(())
    }
}
