use std::sync::Arc;

use yunheshiguang_sdk::connection::Endpoint;
use yunheshiguang_sdk::transport::TransportAdapter;

use crate::workspace::adb::{AdbClient, AdbService, PROBE_POLICY};
use crate::workspace::android_tools::{
    AndroidPlatformToolsRegistry, adb_missing_text, probe_android_tools,
};
use crate::workspace::contract::{
    ConnectionHealth, ConnectionPlan, ConnectionType, DiagnosticCode, DiscoveredEndpoint,
    NetworkEndpoint, PiiClass, UiText,
};
use crate::workspace::messages::*;
use crate::workspace::text::{pii_text, ui_text, ui_text_with_params};

use super::mdns::{MdnsService, ServiceKind, parse_services};

pub const USB_TRANSPORT_ID: &str = "core.usb";
pub const WIRELESS_TRANSPORT_ID: &str = "core.wireless";
pub const OTG_TRANSPORT_ID: &str = "core.otg";

fn plan(id: &str, connection: ConnectionType, title: &str, steps: &[&str]) -> ConnectionPlan {
    ConnectionPlan {
        id: id.to_string(),
        title: ui_text(title).into(),
        connection: connection.into(),
        health: ConnectionHealth::Recovering.into(),
        diagnostic: DiagnosticCode::None.into(),
        steps: steps.iter().map(|key| ui_text(*key)).collect(),
        ..Default::default()
    }
}

pub struct UsbTransport;

impl TransportAdapter for UsbTransport {
    fn plan(&self) -> ConnectionPlan {
        plan(
            USB_TRANSPORT_ID,
            ConnectionType::Usb,
            TRANSPORT_USB_TITLE_KEY,
            &[
                TRANSPORT_USB_STEP1_KEY,
                TRANSPORT_USB_STEP2_KEY,
                TRANSPORT_USB_STEP3_KEY,
                TRANSPORT_USB_STEP4_KEY,
            ],
        )
    }
}

pub struct OtgTransport;

impl TransportAdapter for OtgTransport {
    fn plan(&self) -> ConnectionPlan {
        plan(
            OTG_TRANSPORT_ID,
            ConnectionType::Otg,
            TRANSPORT_OTG_TITLE_KEY,
            &[
                TRANSPORT_OTG_STEP1_KEY,
                TRANSPORT_OTG_STEP2_KEY,
                TRANSPORT_OTG_STEP3_KEY,
                TRANSPORT_OTG_STEP4_KEY,
            ],
        )
    }
}

pub struct WirelessTransport {
    registry: Arc<AndroidPlatformToolsRegistry>,
    adb: Arc<AdbService>,
}

impl WirelessTransport {
    pub fn new(registry: Arc<AndroidPlatformToolsRegistry>, adb: Arc<AdbService>) -> Self {
        Self { registry, adb }
    }

    fn services(&self) -> Result<(Vec<MdnsService>, Vec<String>), UiText> {
        let _runtime = self.adb.acquire_runtime();
        let adb_path = probe_android_tools(&self.registry.config())
            .adb_path
            .ok_or_else(adb_missing_text)?;
        let client = AdbClient::new(adb_path);
        let output = client
            .run(["mdns", "services"], PROBE_POLICY)
            .map_err(|error| {
                ui_text_with_params(
                    TRANSPORT_WIRELESS_DISCOVERY_FAILED_KEY,
                    "",
                    [("message", error.to_string())],
                )
            })?;
        let attached = client
            .devices()
            .map(|devices| devices.into_iter().map(|device| device.serial).collect())
            .unwrap_or_default();

        Ok((parse_services(&output.stdout), attached))
    }
}

fn discovered(service: MdnsService) -> Option<DiscoveredEndpoint> {
    let endpoint = Endpoint::parse(&service.host, service.port).ok()?;

    Some(DiscoveredEndpoint {
        transport_id: String::new(),
        label: pii_text(service.instance, PiiClass::DeviceIdentifier).into(),
        endpoint: NetworkEndpoint {
            host: endpoint.host().to_string(),
            port: u32::from(endpoint.port()),
            ..Default::default()
        }
        .into(),
        requires_pairing: service.kind == ServiceKind::Pairing,
        connection: ConnectionType::Wifi.into(),
        ..Default::default()
    })
}

impl TransportAdapter for WirelessTransport {
    fn plan(&self) -> ConnectionPlan {
        plan(
            WIRELESS_TRANSPORT_ID,
            ConnectionType::Wifi,
            TRANSPORT_WIRELESS_TITLE_KEY,
            &[
                TRANSPORT_WIRELESS_STEP1_KEY,
                TRANSPORT_WIRELESS_STEP2_KEY,
                TRANSPORT_WIRELESS_STEP3_KEY,
                TRANSPORT_WIRELESS_STEP4_KEY,
            ],
        )
    }

    fn discover(&self) -> Result<Vec<DiscoveredEndpoint>, UiText> {
        let (services, attached) = self.services()?;

        Ok(services
            .into_iter()
            .filter(|service| {
                service.kind == ServiceKind::Pairing || !service.is_attached(&attached)
            })
            .filter_map(discovered)
            .collect())
    }
}
