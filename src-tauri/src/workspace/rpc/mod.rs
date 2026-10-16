mod bridge;

use std::fmt::Display;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use connectrpc::error::ErrorDetail;
use connectrpc::{ConnectError, ConnectRpcService, ErrorCode, RequestContext, Router};
use tauri::{App, Manager, Runtime};
use yunheshiguang_contract::buffa::MessageName;
use yunheshiguang_sdk::api::ApiIdentity;

use crate::workspace::contract::UiText;
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

pub use bridge::{RpcStreams, rpc, rpc_cancel};

#[derive(Clone, Debug)]
pub enum Caller {
    Window(String),
    Api(ApiIdentity),
}

impl Caller {
    pub fn of(ctx: &RequestContext) -> Result<Self, ConnectError> {
        ctx.extensions()
            .get::<Self>()
            .cloned()
            .ok_or_else(|| ConnectError::new(ErrorCode::Unauthenticated, "the caller is unknown"))
    }

    pub fn principal(&self, governance: &Governance) -> Principal {
        match self {
            Self::Window(label) => governance.principal(label.as_str()),
            Self::Api(identity) => governance.api_principal(identity),
        }
    }

    pub fn window(&self) -> Option<&str> {
        match self {
            Self::Window(label) => Some(label),
            Self::Api(_) => None,
        }
    }
}

pub fn principal(governance: &Governance, ctx: &RequestContext) -> Result<Principal, ConnectError> {
    Ok(Caller::of(ctx)?.principal(governance))
}

pub fn failure(message: impl Display) -> ConnectError {
    ConnectError::new(ErrorCode::Unavailable, message.to_string())
}

pub fn refusal(code: ErrorCode, text: UiText) -> ConnectError {
    let message = if text.fallback.is_empty() {
        text.key.clone()
    } else {
        text.fallback.clone()
    };

    ConnectError::new(code, message)
        .with_detail(ErrorDetail::from_message(UiText::FULL_NAME, &text))
}

pub struct RpcRoutes {
    pending: Mutex<Option<Router>>,
    service: OnceLock<ConnectRpcService<Router>>,
}

impl RpcRoutes {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(Some(Router::new())),
            service: OnceLock::new(),
        }
    }

    pub fn add(&self, register: impl FnOnce(Router) -> Router) {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let router = pending
            .take()
            .expect("services are added while the workspace is set up");
        *pending = Some(register(router));
    }

    pub fn service(&self) -> ConnectRpcService<Router> {
        self.service
            .get_or_init(|| {
                let router = self
                    .pending
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .take()
                    .unwrap_or_default();
                ConnectRpcService::new(router)
            })
            .clone()
    }
}

pub struct RpcModule;

impl<R: Runtime> WorkspaceModule<R> for RpcModule {
    fn id(&self) -> &'static str {
        "rpc"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::new(RpcRoutes::new()));
        app.manage(Arc::new(RpcStreams::default()));

        Ok(())
    }
}

pub fn serve<R: Runtime>(
    app: &App<R>,
    register: impl FnOnce(Router) -> Router,
) -> Result<(), ModuleSetupError> {
    crate::workspace::module::dependency::<RpcRoutes, R>(app)?.add(register);

    Ok(())
}
