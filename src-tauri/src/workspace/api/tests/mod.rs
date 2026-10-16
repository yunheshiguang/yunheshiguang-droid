use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use connectrpc::{RequestContext, Response as RpcResponse, ServiceRequest, ServiceResult};
use tower::ServiceExt;
use yunheshiguang_contract::yunheshiguang::workspace::v1::{
    BatchActionRequest, BatchActionResult, BatchService, GetWorkspaceSnapshotRequest,
    WorkspaceService, WorkspaceSnapshot,
};
use yunheshiguang_sdk::api::{ApiAuthenticator, ApiAuthenticatorInfo, ApiIdentity};

use super::auth::{Authenticators, CORE_TOKENS_ID, CoreTokens};
use super::gateway::ApiGateway;
use super::relay::EventRelay;
use super::server::{ApiContext, router};
use super::tokens::{MAX_TOKENS, TOKEN_PREFIX, TokenStore};
use crate::workspace::contract::*;
use crate::workspace::messages::*;
use crate::workspace::rpc::{Caller, RpcRoutes};
use crate::workspace::text::ui_text;

#[derive(Default)]
struct Callers(Mutex<Vec<String>>);

impl Callers {
    fn names(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

struct FakeSnapshot;

impl WorkspaceService for FakeSnapshot {
    async fn get_workspace_snapshot(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetWorkspaceSnapshotRequest>,
    ) -> ServiceResult<WorkspaceSnapshot> {
        RpcResponse::ok(WorkspaceSnapshot {
            schema_version: "test".to_string(),
            ..WorkspaceSnapshot::default()
        })
    }
}

struct FakeBatch(Arc<Callers>);

impl BatchService for FakeBatch {
    async fn run_batch_action(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, BatchActionRequest>,
    ) -> ServiceResult<BatchActionResult> {
        if let Ok(Caller::Api(identity)) = Caller::of(&ctx) {
            self.0.0.lock().unwrap().push(identity.name);
        }

        RpcResponse::ok(BatchActionResult {
            accepted: !request.to_owned_message().device_ids.is_empty(),
            ..BatchActionResult::default()
        })
    }
}

struct EnterpriseKeys;

impl ApiAuthenticator for EnterpriseKeys {
    fn describe(&self) -> ApiAuthenticatorInfo {
        ApiAuthenticatorInfo {
            id: "test.enterprise".to_string(),
            ..ApiAuthenticatorInfo::default()
        }
    }

    fn authenticate(&self, token: &str) -> Option<ApiIdentity> {
        (token == "enterprise-key").then(|| ApiIdentity::new("ops").with_roles(["operator"]))
    }
}

struct Api {
    context: ApiContext,
    callers: Arc<Callers>,
    secret: String,
}

fn api() -> Api {
    let tokens = Arc::new(TokenStore::in_memory());
    let (_, secret) = tokens.create("ci").unwrap();
    let callers = Arc::new(Callers::default());
    let rpc = Arc::new(RpcRoutes::new());
    let batch = FakeBatch(Arc::clone(&callers));
    rpc.add(|router| {
        router
            .add_service(Arc::new(FakeSnapshot))
            .add_service(Arc::new(batch))
    });
    let context = ApiContext {
        rpc,
        authenticators: Arc::new(Authenticators::new(
            vec![Box::new(EnterpriseKeys)],
            vec![Box::new(CoreTokens(tokens))],
        )),
        relay: Arc::new(EventRelay::default()),
    };

    Api {
        context,
        callers,
        secret,
    }
}

const SNAPSHOT: &str = "/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot";
const BATCH: &str = "/yunheshiguang.workspace.v1.BatchService/RunBatchAction";

fn call(api: &Api, method: Method, uri: &str, token: Option<&str>, body: &str) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let request = request.body(Body::from(body.to_string())).unwrap();

    tauri::async_runtime::block_on(router(api.context.clone()).oneshot(request)).unwrap()
}

fn json(response: Response) -> serde_json::Value {
    let bytes = tauri::async_runtime::block_on(to_bytes(response.into_body(), 1 << 20)).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

mod lifecycle;
mod routes;
