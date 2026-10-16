use std::sync::Arc;
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_sdk::connection::Endpoint;
use yunheshiguang_sdk::text::public_text;
use yunheshiguang_sdk::transport::TransportAdapter;

use crate::workspace::contract::*;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

use super::service::CloudService;

pub const CLOUD_TRANSPORT_ID: &str = "core.cloud";

pub struct CloudTransport {
    cloud: Arc<CloudService>,
}

impl CloudTransport {
    pub fn new(cloud: Arc<CloudService>) -> Self {
        Self { cloud }
    }
}

pub fn adb_serial(phone: &CloudPhone) -> Option<String> {
    Endpoint::from_contract(phone.endpoint.as_option())
        .ok()
        .map(|endpoint| endpoint.to_string())
}

fn connectable(phone: &CloudPhone) -> bool {
    matches!(
        CloudPhoneStatus::known(phone.status).unwrap_or_default(),
        CloudPhoneStatus::Running | CloudPhoneStatus::Unspecified
    )
}

impl TransportAdapter for CloudTransport {
    fn plan(&self) -> ConnectionPlan {
        ConnectionPlan {
            id: CLOUD_TRANSPORT_ID.to_string(),
            title: ui_text(CLOUD_TRANSPORT_TITLE_KEY).into(),
            connection: ConnectionType::Cloud.into(),
            health: ConnectionHealth::Recovering.into(),
            diagnostic: DiagnosticCode::None.into(),
            steps: [
                CLOUD_TRANSPORT_STEP1_KEY,
                CLOUD_TRANSPORT_STEP2_KEY,
                CLOUD_TRANSPORT_STEP3_KEY,
            ]
            .into_iter()
            .map(ui_text)
            .collect(),
            ..Default::default()
        }
    }

    fn discover(&self) -> Result<Vec<DiscoveredEndpoint>, UiText> {
        let listing = tauri::async_runtime::block_on(self.cloud.list());
        let endpoints = listing
            .phones
            .into_iter()
            .filter(connectable)
            .filter_map(|phone| {
                let endpoint = Endpoint::from_contract(phone.endpoint.as_option()).ok()?;

                Some(DiscoveredEndpoint {
                    transport_id: String::new(),
                    label: public_text(phone.name).into(),
                    endpoint: NetworkEndpoint {
                        host: endpoint.host().to_string(),
                        port: u32::from(endpoint.port()),
                        ..Default::default()
                    }
                    .into(),
                    requires_pairing: false,
                    connection: ConnectionType::Cloud.into(),
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();

        match listing.problems.into_iter().next() {
            Some(problem) if endpoints.is_empty() => {
                Err(problem.problem.into_option().unwrap_or_default())
            }
            _ => Ok(endpoints),
        }
    }
}
