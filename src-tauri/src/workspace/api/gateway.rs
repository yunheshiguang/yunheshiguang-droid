use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Mutex, PoisonError};

use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::workspace::contract::{ApiGatewayConfig, ApiGatewayStatus, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text_with_params;

use super::server::{ApiContext, router};

pub const DEFAULT_PORT: u16 = 47831;
pub const MIN_PORT: u32 = 1024;

pub fn effective_port(config: &ApiGatewayConfig) -> u16 {
    match u16::try_from(config.port) {
        Ok(0) | Err(_) => DEFAULT_PORT,
        Ok(port) => port,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GatewayStatus {
    pub status: ApiGatewayStatus,
    pub address: String,
    pub problem: Option<UiText>,
}

impl GatewayStatus {
    fn stopped() -> Self {
        Self {
            status: ApiGatewayStatus::Stopped,
            address: String::new(),
            problem: None,
        }
    }
}

struct Listening {
    address: SocketAddr,
    shutdown: oneshot::Sender<()>,
}

pub struct ApiGateway {
    context: ApiContext,
    listening: tokio::sync::Mutex<Option<Listening>>,
    status: Mutex<GatewayStatus>,
}

impl ApiGateway {
    pub fn new(context: ApiContext) -> Self {
        Self {
            context,
            listening: tokio::sync::Mutex::new(None),
            status: Mutex::new(GatewayStatus::stopped()),
        }
    }

    pub fn status(&self) -> GatewayStatus {
        self.status
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn set_status(&self, status: GatewayStatus) -> GatewayStatus {
        *self.status.lock().unwrap_or_else(PoisonError::into_inner) = status.clone();
        status
    }

    pub async fn apply(&self, config: &ApiGatewayConfig) -> GatewayStatus {
        let mut listening = self.listening.lock().await;
        let port = effective_port(config);

        if let Some(current) = listening.as_ref()
            && config.enabled
            && current.address.port() == port
        {
            return self.status();
        }
        if let Some(previous) = listening.take() {
            let _ = previous.shutdown.send(());
            self.context.relay.close();
            log::info!("api: stopped listening on {}", previous.address);
        }
        if !config.enabled {
            return self.set_status(GatewayStatus::stopped());
        }

        let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
            Ok(listener) => listener,
            Err(error) => {
                log::warn!("api: cannot listen on port {port}: {error}");
                return self.set_status(GatewayStatus {
                    status: ApiGatewayStatus::Failed,
                    address: String::new(),
                    problem: Some(ui_text_with_params(
                        API_BIND_FAILED_KEY,
                        "",
                        [("message", error.to_string())],
                    )),
                });
            }
        };
        let address = listener
            .local_addr()
            .unwrap_or_else(|_| SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
        let (shutdown, closed) = oneshot::channel::<()>();

        self.context.relay.open();
        let app = router(self.context.clone());
        tauri::async_runtime::spawn(async move {
            let served = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = closed.await;
                })
                .await;

            if let Err(error) = served {
                log::warn!("api: server stopped: {error}");
            }
        });
        log::info!("api: listening on {address}");
        *listening = Some(Listening { address, shutdown });

        self.set_status(GatewayStatus {
            status: ApiGatewayStatus::Running,
            address: format!("http://{address}"),
            problem: None,
        })
    }
}
