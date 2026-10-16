use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::ws::{CloseFrame, Message, WebSocket, close_code};
use axum::extract::{Query, Request, State, WebSocketUpgrade};
use axum::http::header::{AUTHORIZATION, RETRY_AFTER};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::Deserialize;
use tokio::sync::broadcast;
use tower::ServiceExt;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use yunheshiguang_contract::LOCAL_API_PROCEDURES;
use yunheshiguang_sdk::api::ApiIdentity;

use crate::workspace::contract::{ApiEvent, UiText};
use crate::workspace::messages::*;
use crate::workspace::rpc::{Caller, RpcRoutes};
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::auth::Authenticators;
use super::openapi::document;
use super::relay::EventRelay;

const REPLENISH_EVERY_MILLISECONDS: u64 = 100;
const BURST: u32 = 20;
const EVENTS_PATH: &str = "/v1/events";
const RECHECK_EVERY: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct ApiContext {
    pub rpc: Arc<RpcRoutes>,
    pub authenticators: Arc<Authenticators>,
    pub relay: Arc<EventRelay>,
}

#[derive(Clone)]
struct Presented(String);

fn rejected(status: StatusCode, code: &str, text: UiText) -> Response {
    let message = if text.fallback.is_empty() {
        text.key
    } else {
        text.fallback
    };

    (
        status,
        Json(serde_json::json!({ "code": code, "message": message })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct TokenQuery {
    access_token: String,
}

fn presented_token(headers: &HeaderMap, request: &Request) -> Option<String> {
    let bearer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
        .map(|(_, token)| token.trim().to_string());

    bearer.or_else(|| {
        (request.uri().path() == EVENTS_PATH)
            .then(|| Query::<TokenQuery>::try_from_uri(request.uri()).ok())
            .flatten()
            .map(|query| query.0.access_token)
    })
}

async fn identify(authenticators: &Arc<Authenticators>, token: String) -> Option<ApiIdentity> {
    let authenticators = Arc::clone(authenticators);
    tauri::async_runtime::spawn_blocking(move || authenticators.identify(&token))
        .await
        .ok()
        .flatten()
}

async fn authenticate(
    State(context): State<ApiContext>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = presented_token(request.headers(), &request).unwrap_or_default();
    match identify(&context.authenticators, token.clone()).await {
        Some(identity) => {
            request.extensions_mut().insert(Caller::Api(identity));
            request.extensions_mut().insert(Presented(token));
            next.run(request).await
        }
        None => rejected(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            ui_text(API_UNAUTHORIZED_KEY),
        ),
    }
}

#[derive(Clone)]
struct CallerKey;

impl KeyExtractor for CallerKey {
    type Key = String;

    fn extract<T>(&self, request: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
        match request.extensions().get::<Caller>() {
            Some(Caller::Api(identity)) => Ok(identity.name.clone()),
            _ => Err(GovernorError::UnableToExtractKey),
        }
    }
}

fn rate_limited(error: GovernorError) -> Response {
    match error {
        GovernorError::TooManyRequests { wait_time, headers } => {
            let seconds = wait_time.max(1);
            let mut response = rejected(
                StatusCode::TOO_MANY_REQUESTS,
                "resource_exhausted",
                ui_text_with_params(API_RATE_LIMITED_KEY, "", [("seconds", seconds.to_string())]),
            );
            if let Some(headers) = headers {
                response.headers_mut().extend(headers);
            }
            response
                .headers_mut()
                .insert(RETRY_AFTER, HeaderValue::from(seconds));
            response
        }
        _ => rejected(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            ui_text(API_UNAUTHORIZED_KEY),
        ),
    }
}

async fn procedures(State(context): State<ApiContext>, request: Request) -> Response {
    if !LOCAL_API_PROCEDURES.contains(&request.uri().path()) {
        return rejected(
            StatusCode::NOT_FOUND,
            "unimplemented",
            ui_text(API_NOT_FOUND_KEY),
        );
    }

    match context.rpc.service().oneshot(request).await {
        Ok(response) => response.map(Body::new),
        Err(never) => match never {},
    }
}

pub fn router(context: ApiContext) -> Router {
    let limits = GovernorConfigBuilder::default()
        .per_millisecond(REPLENISH_EVERY_MILLISECONDS)
        .burst_size(BURST)
        .key_extractor(CallerKey)
        .finish()
        .expect("rate limits are non-zero");
    let protected = Router::new()
        .route(EVENTS_PATH, get(events))
        .route("/v1/openapi.json", get(openapi))
        .fallback(procedures)
        .layer(GovernorLayer::new(limits).error_handler(rate_limited))
        .layer(middleware::from_fn_with_state(
            context.clone(),
            authenticate,
        ));

    Router::new()
        .route("/v1/health", get(health))
        .merge(protected)
        .with_state(context)
}

async fn health() -> Response {
    Json(serde_json::json!({ "status": "ok" })).into_response()
}

async fn openapi() -> Response {
    Json(document()).into_response()
}

async fn events(
    State(context): State<ApiContext>,
    Extension(presented): Extension<Presented>,
    upgrade: WebSocketUpgrade,
) -> Response {
    match context.relay.subscribe() {
        Some(receiver) => upgrade.on_upgrade(move |socket| {
            forward_events(socket, receiver, context.authenticators, presented.0)
        }),
        None => rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            ui_text_with_params(API_UNAVAILABLE_KEY, "", [("message", "events are closed")]),
        ),
    }
}

async fn forward_events(
    mut socket: WebSocket,
    mut events: broadcast::Receiver<ApiEvent>,
    authenticators: Arc<Authenticators>,
    token: String,
) {
    let mut recheck =
        tokio::time::interval_at(tokio::time::Instant::now() + RECHECK_EVERY, RECHECK_EVERY);
    loop {
        tokio::select! {
            _ = recheck.tick() => {
                if identify(&authenticators, token.clone()).await.is_none() {
                    let close = CloseFrame {
                        code: close_code::POLICY,
                        reason: "unauthorized".into(),
                    };
                    let _ = socket.send(Message::Close(Some(close))).await;
                    break;
                }
            }
            event = events.recv() => match event {
                Ok(event) => {
                    let Ok(text) = serde_json::to_string(&event) else {
                        continue;
                    };
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    log::debug!("api: a slow event client skipped {skipped} event(s)");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
        }
    }
}
