use std::sync::{Arc, Mutex};

use tempfile::TempDir;
use yunheshiguang_sdk::cloud::{CloudFuture, CloudPhoneProvider};
use yunheshiguang_sdk::documents::FolderDocumentStore;
use yunheshiguang_sdk::transport::TransportAdapter;

use super::registered::{REGISTERED_PROVIDER_ID, RegisteredPhones};
use super::section::CloudSection;
use super::service::{CLOUD_PHONES_CHANGED_EVENT, CloudService};
use super::transport::CloudTransport;
use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::*;
use crate::workspace::events::WorkspaceEvents;
use crate::workspace::governance::Governance;
use crate::workspace::messages::*;
use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::ui_text;

fn endpoint(host: &str, port: u32) -> NetworkEndpoint {
    NetworkEndpoint {
        host: host.to_string(),
        port,
        ..Default::default()
    }
}

fn phone(id: &str, status: CloudPhoneStatus, host: &str) -> CloudPhone {
    CloudPhone {
        id: id.to_string(),
        name: format!("Redroid {id}"),
        status: status.into(),
        endpoint: endpoint(host, 5555).into(),
        ..CloudPhone::default()
    }
}

struct Farm {
    phones: Mutex<Vec<CloudPhone>>,
    down: bool,
}

impl Farm {
    fn with(phones: Vec<CloudPhone>) -> Arc<Self> {
        Arc::new(Self {
            phones: Mutex::new(phones),
            down: false,
        })
    }

    fn set(&self, id: &str, status: CloudPhoneStatus) -> Result<CloudPhone, UiText> {
        let mut phones = self.phones.lock().unwrap();
        let phone = phones
            .iter_mut()
            .find(|phone| phone.id == id)
            .ok_or_else(|| ui_text(CLOUD_PHONE_MISSING_KEY))?;
        phone.status = status.into();
        Ok(phone.clone())
    }
}

impl CloudPhoneProvider for Farm {
    fn describe(&self) -> CloudProviderInfo {
        CloudProviderInfo {
            id: "test.farm".to_string(),
            can_start: true,
            can_stop: true,
            ..CloudProviderInfo::default()
        }
    }

    fn phones(&self) -> CloudFuture<'_, Vec<CloudPhone>> {
        let listed = if self.down {
            Err(ui_text("test.down"))
        } else {
            Ok(self.phones.lock().unwrap().clone())
        };
        Box::pin(async move { listed })
    }

    fn start<'a>(&'a self, id: &'a str) -> CloudFuture<'a, CloudPhone> {
        Box::pin(async move { self.set(id, CloudPhoneStatus::Running) })
    }

    fn stop<'a>(&'a self, id: &'a str) -> CloudFuture<'a, CloudPhone> {
        Box::pin(async move { self.set(id, CloudPhoneStatus::Stopped) })
    }
}

#[derive(Default)]
struct RecordingEvents(Mutex<Vec<String>>);

impl WorkspaceEvents for RecordingEvents {
    fn emit(&self, event: &'static str, _payload: serde_json::Value) {
        self.0.lock().unwrap().push(event.to_string());
    }
}

struct Harness {
    service: Arc<CloudService>,
    governance: Arc<Governance>,
    events: Arc<RecordingEvents>,
    _documents: TempDir,
}

fn harness(extension: Vec<Arc<dyn CloudPhoneProvider>>) -> Harness {
    let directory = tempfile::tempdir().unwrap();
    let registered = Arc::new(RegisteredPhones::new(Arc::new(FolderDocumentStore::new(
        directory.path(),
    ))));
    let mut providers = extension;
    providers.push(Arc::clone(&registered) as Arc<dyn CloudPhoneProvider>);
    let governance = Arc::new(Governance::core());
    let events = Arc::new(RecordingEvents::default());

    Harness {
        service: Arc::new(CloudService {
            providers,
            registered,
            governance: Arc::clone(&governance),
            telemetry: Arc::new(Telemetry::assemble(Vec::new())),
            events: Arc::clone(&events) as Arc<dyn WorkspaceEvents>,
            cache: Mutex::new(Vec::new()),
        }),
        governance,
        events,
        _documents: directory,
    }
}

fn request(provider_id: &str, phone_id: &str) -> CloudPhoneRequest {
    CloudPhoneRequest {
        provider_id: provider_id.to_string(),
        phone_id: phone_id.to_string(),
        scope: Default::default(),
        ..Default::default()
    }
}

impl Harness {
    fn register(&self, name: &str, host: &str, port: u32) -> CloudPhoneResult {
        tauri::async_runtime::block_on(self.service.register(
            RegisterCloudPhoneRequest {
                name: name.to_string(),
                endpoint: endpoint(host, port).into(),
                scope: Default::default(),
                ..Default::default()
            },
            &self.governance.principal("main"),
        ))
    }

    fn audited(&self) -> Vec<AuditEntry> {
        self.governance.state().audit_trail
    }
}

#[test]
fn registers_phones_by_address_once_and_removes_them() {
    let harness = harness(Vec::new());

    let registered = harness.register(" Cloud 01 ", "10.20.0.11", 5555);
    assert!(registered.accepted, "{:?}", registered.detail);
    let phone = registered.phone.unwrap();
    assert_eq!(
        (phone.provider_id.as_str(), phone.name.as_str()),
        (REGISTERED_PROVIDER_ID, "Cloud 01")
    );

    assert_eq!(
        harness.register("Again", "10.20.0.11", 5555).detail,
        ui_text(CLOUD_ALREADY_REGISTERED_KEY).into()
    );
    assert_eq!(
        harness.register("Bad", "not a host", 5555).detail,
        ui_text(CLOUD_ENDPOINT_INVALID_KEY).into()
    );
    assert_eq!(
        harness
            .register("  ", "10.20.0.12", 5555)
            .detail
            .unwrap()
            .key,
        CLOUD_NAME_INVALID_KEY
    );

    let listed = tauri::async_runtime::block_on(harness.service.list());
    assert_eq!(listed.phones.len(), 1);
    assert_eq!(harness.service.cached(), listed.phones);

    let removed = tauri::async_runtime::block_on(harness.service.remove(
        request(REGISTERED_PROVIDER_ID, &phone.id),
        &harness.governance.principal("main"),
    ));
    assert!(removed.accepted);
    assert!(
        tauri::async_runtime::block_on(harness.service.list())
            .phones
            .is_empty()
    );

    let audited = harness.audited();
    assert_eq!(
        audited.len(),
        3,
        "a register, the refused duplicate, and a removal"
    );
    assert!(
        audited
            .iter()
            .all(|entry| entry.action == GovernedAction::ManageCloudPhones)
    );
    assert_eq!(audited[2].targets, ["***5555"]);
    assert_eq!(
        harness.events.0.lock().unwrap().as_slice(),
        [CLOUD_PHONES_CHANGED_EVENT, CLOUD_PHONES_CHANGED_EVENT]
    );
}

#[test]
fn lists_every_provider_and_keeps_the_failures_apart() {
    let farm = Farm::with(vec![phone("a", CloudPhoneStatus::Stopped, "10.0.0.1")]);
    let down = Arc::new(Farm {
        phones: Mutex::new(Vec::new()),
        down: true,
    });
    let harness = harness(vec![farm, down]);

    let listed = tauri::async_runtime::block_on(harness.service.list());

    assert_eq!(listed.providers.len(), 3);
    assert_eq!(listed.phones[0].provider_id, "test.farm");
    assert_eq!(listed.problems.len(), 1);
    assert_eq!(listed.problems[0].problem, ui_text("test.down").into());
}

#[test]
fn starts_and_stops_through_providers_that_can() {
    let farm = Farm::with(vec![phone("a", CloudPhoneStatus::Stopped, "10.0.0.1")]);
    let harness = harness(vec![farm]);
    tauri::async_runtime::block_on(harness.service.list());
    let principal = harness.governance.principal("main");

    let started = tauri::async_runtime::block_on(
        harness.service.start(request("test.farm", "a"), &principal),
    );
    assert_eq!(started.phone.unwrap().status, CloudPhoneStatus::Running);
    assert_eq!(
        harness.service.cached()[0].status,
        CloudPhoneStatus::Running
    );
    let stopped =
        tauri::async_runtime::block_on(harness.service.stop(request("test.farm", "a"), &principal));
    assert_eq!(stopped.phone.unwrap().status, CloudPhoneStatus::Stopped);

    let unsupported = tauri::async_runtime::block_on(
        harness
            .service
            .start(request(REGISTERED_PROVIDER_ID, "x"), &principal),
    );
    assert_eq!(unsupported.detail, ui_text(CLOUD_UNSUPPORTED_KEY).into());
    let unknown =
        tauri::async_runtime::block_on(harness.service.stop(request("nobody", "x"), &principal));
    assert_eq!(unknown.detail, ui_text(CLOUD_PROVIDER_UNKNOWN_KEY).into());

    let audited = harness.audited();
    assert_eq!(audited.len(), 2, "refused requests never reach governance");
    assert_eq!(
        audited[1].detail.as_option().unwrap().key,
        CLOUD_AUDIT_STARTED_KEY
    );
    assert_eq!(audited[1].targets, ["***5555"]);
}

#[test]
fn offers_running_phones_for_discovery_and_marks_them_on_the_wall() {
    let farm = Farm::with(vec![
        phone("on", CloudPhoneStatus::Running, "10.0.0.1"),
        phone("off", CloudPhoneStatus::Stopped, "10.0.0.2"),
    ]);
    let harness = harness(vec![farm]);
    let transport = CloudTransport::new(Arc::clone(&harness.service));

    let found = tauri::async_runtime::block_on(run_blocking(move || Ok(transport.discover())))
        .unwrap()
        .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].connection, ConnectionType::Cloud);
    assert_eq!(found[0].endpoint, endpoint("10.0.0.1", 5555).into());

    let mut draft = SnapshotDraft::new("test", "zh".to_string(), "macos".to_string());
    draft.add_devices(vec![
        DeviceRecord {
            id: "10.0.0.1:5555".to_string(),
            connection: ConnectionType::Wifi.into(),
            ..DeviceRecord::default()
        },
        DeviceRecord {
            id: "R58M20ABCDE".to_string(),
            connection: ConnectionType::Usb.into(),
            ..DeviceRecord::default()
        },
    ]);
    CloudSection::new(&harness.service.cached()).contribute(&mut draft);
    let connections = draft
        .devices()
        .iter()
        .map(|device| device.connection)
        .collect::<Vec<_>>();
    assert_eq!(connections, [ConnectionType::Cloud, ConnectionType::Usb]);
}

#[test]
fn reports_discovery_as_failed_only_when_nothing_could_be_listed() {
    let down = Arc::new(Farm {
        phones: Mutex::new(Vec::new()),
        down: true,
    });
    let harness = harness(vec![down]);
    let transport = CloudTransport::new(Arc::clone(&harness.service));

    assert_eq!(
        tauri::async_runtime::block_on(run_blocking(move || Ok(transport.discover()))).unwrap(),
        Err(ui_text("test.down"))
    );
}

#[test]
fn shares_the_unsupported_key_with_the_sdk() {
    assert_eq!(
        CLOUD_UNSUPPORTED_KEY,
        yunheshiguang_sdk::cloud::CLOUD_UNSUPPORTED_KEY
    );
}
