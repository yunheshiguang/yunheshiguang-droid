use yunheshiguang_sdk::transport::TransportAdapter;

use super::builtin::{OTG_TRANSPORT_ID, OtgTransport, USB_TRANSPORT_ID, UsbTransport};
use super::mdns::{MdnsService, ServiceKind, parse_services};
use super::registry::TransportRegistry;
use crate::workspace::contract::{
    ConnectionPlan, ConnectionType, DiscoveredEndpoint, NetworkEndpoint, UiText,
};
use crate::workspace::text::ui_text;

#[test]
fn reads_the_services_adb_discovered_and_skips_everything_else() {
    let output = b"List of discovered mdns services\n\
adb-R58M20ABCDE-gO3wVN\t_adb-tls-connect._tcp\t192.168.1.23:37115\n\
adb-R58M20ABCDE-gO3wVN\t_adb-tls-pairing._tcp.\t192.168.1.23:41235\n\
adb-emulator\t_adb._tcp\t[fe80::1]:5555\n\
printer\t_ipp._tcp\t192.168.1.9:631\n\
\n";

    assert_eq!(
        parse_services(output),
        vec![
            MdnsService {
                instance: "adb-R58M20ABCDE-gO3wVN".to_string(),
                kind: ServiceKind::Connect,
                host: "192.168.1.23".to_string(),
                port: 37115,
            },
            MdnsService {
                instance: "adb-R58M20ABCDE-gO3wVN".to_string(),
                kind: ServiceKind::Pairing,
                host: "192.168.1.23".to_string(),
                port: 41235,
            },
            MdnsService {
                instance: "adb-emulator".to_string(),
                kind: ServiceKind::Legacy,
                host: "fe80::1".to_string(),
                port: 5555,
            },
        ]
    );
    assert!(parse_services(b"List of discovered mdns services\n\n").is_empty());
}

#[test]
fn keeps_instance_names_that_contain_spaces() {
    let renamed = parse_services(b"Pixel 7 (2)\t_adb-tls-connect._tcp\t192.168.1.40:40001\n");

    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0].instance, "Pixel 7 (2)");
    assert_eq!(renamed[0].port, 40001);
}

#[test]
fn recognizes_devices_adb_already_attached_by_either_name() {
    let service = MdnsService {
        instance: "adb-R58M20ABCDE-gO3wVN".to_string(),
        kind: ServiceKind::Connect,
        host: "192.168.1.23".to_string(),
        port: 37115,
    };

    assert!(service.is_attached(&["adb-R58M20ABCDE-gO3wVN._adb-tls-connect._tcp".to_string()]));
    assert!(service.is_attached(&["192.168.1.23:37115".to_string()]));
    assert!(!service.is_attached(&["adb-R58M20ABCDE-other._adb-tls-connect._tcp".to_string()]));
}

struct Inventory {
    id: &'static str,
    found: Result<Vec<DiscoveredEndpoint>, UiText>,
}

impl TransportAdapter for Inventory {
    fn plan(&self) -> ConnectionPlan {
        ConnectionPlan {
            id: self.id.to_string(),
            connection: ConnectionType::Cloud.into(),
            ..ConnectionPlan::default()
        }
    }

    fn discover(&self) -> Result<Vec<DiscoveredEndpoint>, UiText> {
        self.found.clone()
    }
}

fn endpoint(host: &str) -> DiscoveredEndpoint {
    DiscoveredEndpoint {
        endpoint: NetworkEndpoint {
            host: host.to_string(),
            port: 5555,
            ..Default::default()
        }
        .into(),
        ..DiscoveredEndpoint::default()
    }
}

#[test]
fn keeps_core_plans_first_and_the_first_transport_for_each_id() {
    let registry = TransportRegistry::new(
        vec![Box::new(UsbTransport), Box::new(OtgTransport)],
        vec![
            Box::new(Inventory {
                id: "cloud",
                found: Ok(Vec::new()),
            }),
            Box::new(Inventory {
                id: USB_TRANSPORT_ID,
                found: Ok(vec![endpoint("10.0.0.1")]),
            }),
        ],
    );

    let ids = registry
        .plans()
        .into_iter()
        .map(|plan| plan.id)
        .collect::<Vec<_>>();

    assert_eq!(ids, vec![USB_TRANSPORT_ID, OTG_TRANSPORT_ID, "cloud"]);
    assert!(registry.discover().endpoints.is_empty());
}

#[test]
fn stamps_each_endpoint_with_its_transport_and_isolates_failures() {
    let registry = TransportRegistry::new(
        vec![Box::new(UsbTransport)],
        vec![
            Box::new(Inventory {
                id: "cloud",
                found: Ok(vec![endpoint("10.0.0.1"), endpoint("10.0.0.2")]),
            }),
            Box::new(Inventory {
                id: "relay",
                found: Err(ui_text("relay.offline")),
            }),
        ],
    );

    let discovery = registry.discover();

    assert_eq!(
        discovery
            .endpoints
            .iter()
            .map(|endpoint| endpoint.transport_id.as_str())
            .collect::<Vec<_>>(),
        vec!["cloud", "cloud"]
    );
    assert_eq!(discovery.failures.len(), 1);
    assert_eq!(discovery.failures[0].transport_id, "relay");
    assert_eq!(
        discovery.failures[0]
            .reason
            .as_option()
            .map(|reason| reason.key.as_str()),
        Some("relay.offline")
    );
}
