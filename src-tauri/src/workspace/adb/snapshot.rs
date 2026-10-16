use crate::workspace::contract::*;
use crate::workspace::diagnostics::signal;
use crate::workspace::messages::*;
use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};
use crate::workspace::text::pii_text;

use super::transport::classify;
use super::{AdbDevice, AdbEnvironment, AdbProbeDiagnostic};

pub struct AdbSection<'a> {
    environment: &'a AdbEnvironment,
    adb_path: String,
}

impl<'a> AdbSection<'a> {
    pub fn new(environment: &'a AdbEnvironment, adb_path: String) -> Self {
        Self {
            environment,
            adb_path,
        }
    }
}

impl SnapshotSection for AdbSection<'_> {
    fn id(&self) -> &'static str {
        "adb"
    }

    fn contribute(&self, draft: &mut SnapshotDraft) {
        let devices = self
            .environment
            .devices
            .iter()
            .enumerate()
            .map(|(index, device)| to_device_record(device, (index + 1) as u32))
            .collect::<Vec<_>>();

        draft.add_devices(devices);
        draft.add_diagnostics(
            self.environment
                .diagnostics
                .iter()
                .enumerate()
                .map(|(index, item)| to_diagnostic_signal(index, item)),
        );
        draft.set_adb_runtime(runtime_state(&self.adb_path, self.environment));
    }
}

fn to_device_record(device: &AdbDevice, selected_rank: u32) -> DeviceRecord {
    let facts = device.state.facts();
    let model = device
        .model
        .clone()
        .or_else(|| device.device.clone())
        .unwrap_or_default();

    DeviceRecord {
        id: device.serial.clone(),
        name: pii_text(device.serial.clone(), PiiClass::DeviceIdentifier).into(),
        model: if model.is_empty() {
            pii_text(
                DEVICE_MODEL_FALLBACK_VALUE.to_string(),
                PiiClass::DeviceModel,
            )
        } else {
            pii_text(model, PiiClass::DeviceModel)
        }
        .into(),
        group: Default::default(),
        status: facts.status.into(),
        battery_percent: None,
        fps: None,
        latency_ms: None,
        connection: classify(device).connection_type().into(),
        note: device.state.note().into(),
        tags: Vec::new(),
        health: facts.health.into(),
        diagnostic: facts.diagnostic.into(),
        selected_rank,
        group_id: String::new(),
        labels: Default::default(),
        ..Default::default()
    }
}

fn to_diagnostic_signal(index: usize, item: &AdbProbeDiagnostic) -> DiagnosticSignal {
    signal(
        format!("adb-diagnostic-{index}"),
        item.code,
        item.message.clone(),
    )
}

fn runtime_state(adb_path: &str, environment: &AdbEnvironment) -> AdbRuntimeState {
    let devices = environment
        .devices
        .iter()
        .map(|device| AdbDeviceContract {
            serial: device.serial.clone(),
            state: device.state.facts().contract.into(),
            transport: classify(device).kind().into(),
            model: device.model.clone().unwrap_or_default(),
            device: device.device.clone().unwrap_or_default(),
            product: device.product.clone().unwrap_or_default(),
            usb: device.usb.clone().unwrap_or_default(),
            transport_id: device.transport_id.clone().unwrap_or_default(),
            ..Default::default()
        })
        .collect();

    AdbRuntimeState {
        executable_path: adb_path.to_string(),
        version: environment.version.clone().unwrap_or_default(),
        server_state: if environment.version.is_some() {
            AdbServerState::Ready.into()
        } else {
            AdbServerState::Unavailable.into()
        },
        devices,
        ..Default::default()
    }
}
