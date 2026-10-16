use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use bytes::Bytes;
use connectrpc::ConnectRpcBody;
use futures_util::future::{AbortHandle, AbortRegistration, Abortable};
use http::{HeaderMap, Method, Request, header::CONTENT_TYPE};
use http_body_util::{BodyExt, Full};
use serde::Serialize;
use tauri::ipc::{Channel, InvokeBody, InvokeResponseBody, JavaScriptChannelId};
use tauri::{State, Webview};
use tower::ServiceExt;

use super::{Caller, RpcRoutes};

const PATH_HEADER: &str = "x-rpc-path";
const CHANNEL_HEADER: &str = "x-rpc-channel";
const CONTENT_TYPE_HEADER: &str = "x-rpc-content-type";
const BRIDGE_HEADER_PREFIX: &str = "x-rpc-";

type StreamKey = (String, u32);

#[derive(Default)]
pub struct RpcStreams(Mutex<HashMap<StreamKey, AbortHandle>>);

impl RpcStreams {
    fn lock(&self) -> MutexGuard<'_, HashMap<StreamKey, AbortHandle>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn open(&self, key: StreamKey) -> AbortRegistration {
        let (handle, registration) = AbortHandle::new_pair();
        if let Some(previous) = self.lock().insert(key, handle) {
            previous.abort();
        }
        registration
    }

    fn close(&self, key: &StreamKey) -> Option<AbortHandle> {
        self.lock().remove(key)
    }

    fn cancel(&self, window: &str, channel: u32) -> bool {
        self.close(&(window.to_string(), channel))
            .map(|stream| stream.abort())
            .is_some()
    }
}

#[derive(Serialize)]
pub struct RpcHead {
    status: u16,
    headers: Vec<(String, String)>,
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| format!("the {name} header is missing"))
}

async fn forward(mut body: ConnectRpcBody, channel: Channel<InvokeResponseBody>) {
    loop {
        let message = match body.frame().await {
            Some(Ok(frame)) => match frame.into_data() {
                Ok(data) => InvokeResponseBody::Raw(data.to_vec()),
                Err(_) => continue,
            },
            Some(Err(error)) => InvokeResponseBody::Json(
                serde_json::json!({ "error": error.to_string() }).to_string(),
            ),
            None => InvokeResponseBody::Json(r#"{"end":true}"#.to_string()),
        };
        let last = matches!(message, InvokeResponseBody::Json(_));
        if channel.send(message).is_err() || last {
            return;
        }
    }
}

async fn dispatch(
    routes: &RpcRoutes,
    headers: &HeaderMap,
    body: Bytes,
    caller: Caller,
) -> Result<(RpcHead, ConnectRpcBody), String> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(header(headers, PATH_HEADER)?)
        .header(CONTENT_TYPE, header(headers, CONTENT_TYPE_HEADER)?);
    for (name, value) in headers {
        if name != CONTENT_TYPE && !name.as_str().starts_with(BRIDGE_HEADER_PREFIX) {
            builder = builder.header(name, value);
        }
    }
    let mut forwarded = builder
        .body(Full::new(body))
        .map_err(|error| error.to_string())?;
    forwarded.extensions_mut().insert(caller);

    let response = match routes.service().oneshot(forwarded).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    let (parts, body) = response.into_parts();
    let head = RpcHead {
        status: parts.status.as_u16(),
        headers: parts
            .headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (name.to_string(), value.to_string()))
            })
            .collect(),
    };

    Ok((head, body))
}

fn request_bytes(body: &InvokeBody) -> Result<Bytes, String> {
    match body {
        InvokeBody::Raw(bytes) => Ok(Bytes::copy_from_slice(bytes)),
        InvokeBody::Json(value) => serde_json::from_value::<Vec<u8>>(value.clone())
            .map(Bytes::from)
            .map_err(|_| "the request body is not binary".to_string()),
    }
}

#[tauri::command]
pub async fn rpc(
    request: tauri::ipc::Request<'_>,
    webview: Webview,
    routes: State<'_, Arc<RpcRoutes>>,
    streams: State<'_, Arc<RpcStreams>>,
) -> Result<RpcHead, String> {
    let headers = request.headers();
    let channel: Channel<InvokeResponseBody> = header(headers, CHANNEL_HEADER)?
        .parse::<JavaScriptChannelId>()
        .map_err(|error| error.to_string())?
        .channel_on(webview.clone());
    let body = request_bytes(request.body())?;

    let (head, body) = dispatch(
        &routes,
        headers,
        body,
        Caller::Window(webview.label().to_string()),
    )
    .await?;
    let key = (webview.label().to_string(), channel.id());
    let registration = streams.open(key.clone());
    let streams = Arc::clone(&streams);
    tauri::async_runtime::spawn(async move {
        let _ = Abortable::new(forward(body, channel), registration).await;
        streams.close(&key);
    });

    Ok(head)
}

#[tauri::command]
pub async fn rpc_cancel(
    channel: u32,
    webview: Webview,
    streams: State<'_, Arc<RpcStreams>>,
) -> Result<(), String> {
    streams.cancel(webview.label(), channel);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
    use http::HeaderValue;
    use http_body_util::BodyExt;
    use yunheshiguang_contract::Message;
    use yunheshiguang_contract::yunheshiguang::workspace::v1::{
        GetWorkspaceSnapshotRequest, WorkspaceService, WorkspaceSnapshot,
    };

    use super::*;

    #[derive(Default)]
    struct Snapshots(Mutex<Vec<String>>);

    impl WorkspaceService for Snapshots {
        async fn get_workspace_snapshot(
            &self,
            ctx: RequestContext,
            _request: ServiceRequest<'_, GetWorkspaceSnapshotRequest>,
        ) -> ServiceResult<WorkspaceSnapshot> {
            if let Ok(Caller::Window(label)) = Caller::of(&ctx) {
                self.0.lock().unwrap().push(label);
            }

            Response::ok(WorkspaceSnapshot {
                schema_version: "bridged".to_string(),
                ..WorkspaceSnapshot::default()
            })
        }
    }

    #[test]
    fn hands_a_window_call_to_its_service_and_returns_the_reply() {
        let snapshots = Arc::new(Snapshots::default());
        let routes = RpcRoutes::new();
        let service = Arc::clone(&snapshots);
        routes.add(|router| router.add_service(service));
        let mut headers = HeaderMap::new();
        headers.insert(
            PATH_HEADER,
            HeaderValue::from_static(
                "/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot",
            ),
        );
        headers.insert(CHANNEL_HEADER, HeaderValue::from_static("__CHANNEL__:7"));
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/octet-stream"),
        );
        headers.insert(
            CONTENT_TYPE_HEADER,
            HeaderValue::from_static("application/proto"),
        );
        headers.insert("connect-protocol-version", HeaderValue::from_static("1"));
        let request = Bytes::from(GetWorkspaceSnapshotRequest::default().encode_to_vec());

        let (head, body) = tauri::async_runtime::block_on(dispatch(
            &routes,
            &headers,
            request,
            Caller::Window("main".to_string()),
        ))
        .unwrap();
        let bytes = tauri::async_runtime::block_on(body.collect())
            .unwrap()
            .to_bytes();

        assert_eq!(head.status, 200);
        assert!(
            head.headers
                .iter()
                .any(|(name, value)| name == "content-type" && value == "application/proto")
        );
        assert_eq!(
            WorkspaceSnapshot::decode_from_slice(&bytes)
                .unwrap()
                .schema_version,
            "bridged"
        );
        assert_eq!(*snapshots.0.lock().unwrap(), ["main"]);
    }

    #[test]
    fn reads_bodies_sent_as_bytes_or_as_a_json_array() {
        assert_eq!(
            request_bytes(&InvokeBody::Raw(vec![8, 1])).unwrap(),
            Bytes::from_static(&[8, 1])
        );
        assert_eq!(
            request_bytes(&InvokeBody::Json(serde_json::json!([8, 1]))).unwrap(),
            Bytes::from_static(&[8, 1])
        );
        assert!(request_bytes(&InvokeBody::Json(serde_json::json!({ "a": 1 }))).is_err());
    }

    #[test]
    fn stops_a_stream_only_for_the_window_that_opened_it() {
        let streams = RpcStreams::default();
        let registration = streams.open(("main".to_string(), 7));
        let forwarding = Abortable::new(std::future::pending::<()>(), registration);

        assert!(!streams.cancel("workspace_2", 7));
        assert!(streams.cancel("main", 7));
        assert!(!streams.cancel("main", 7));
        assert!(tauri::async_runtime::block_on(forwarding).is_err());
    }

    #[test]
    fn refuses_a_call_without_a_procedure() {
        let headers = HeaderMap::new();

        let refused = tauri::async_runtime::block_on(dispatch(
            &RpcRoutes::new(),
            &headers,
            Bytes::new(),
            Caller::Window("main".to_string()),
        ));

        assert!(refused.is_err());
    }
}
