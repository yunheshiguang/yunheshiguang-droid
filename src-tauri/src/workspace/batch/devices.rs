use crate::workspace::adb::{AdbDevice, AdbDeviceState, AdbProbeDiagnostic, AdbService};
use crate::workspace::contract::*;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

use super::errors::{diagnostic_error_kind, diagnostic_message};
use super::task::{diagnostics_from, task_for_request};

pub(super) fn plan_devices(
    devices: &[AdbDevice],
    device_ids: &[String],
    allowed_states: &[AdbDeviceState],
    adb_service: &AdbService,
) -> (Vec<String>, Vec<AdbDeviceActionResult>) {
    let mut runnable = Vec::new();
    let mut failures = Vec::new();

    for device_id in device_ids {
        match devices.iter().find(|device| device.serial == *device_id) {
            Some(device) if allowed_states.contains(&device.state) => {
                runnable.push(device.serial.clone());
            }
            Some(device) => {
                let diagnostic = adb_service
                    .diagnostic_for_device(device)
                    .unwrap_or_else(device_not_available);
                failures.push(failed_device_result(device_id, &diagnostic));
            }
            None => {
                failures.push(failed_device_result(device_id, &device_not_available()));
            }
        }
    }

    (runnable, failures)
}

fn device_not_available() -> AdbProbeDiagnostic {
    AdbProbeDiagnostic {
        code: DiagnosticCode::DeviceOffline,
        message: ui_text(VALIDATION_DEVICE_NOT_CONNECTED_KEY),
    }
}

pub(super) fn failed_device_result(
    device_id: &str,
    diagnostic: &AdbProbeDiagnostic,
) -> AdbDeviceActionResult {
    AdbDeviceActionResult {
        device_id: device_id.to_string(),
        success: false,
        error_kind: diagnostic_error_kind(diagnostic.code).into(),
        message: diagnostic_message(diagnostic),
        ..Default::default()
    }
}

pub(super) fn unavailable_result(
    request: &BatchActionRequest,
    diagnostic: AdbProbeDiagnostic,
) -> BatchActionResult {
    let error_kind = diagnostic_error_kind(diagnostic.code);
    let message = diagnostic_message(&diagnostic);

    BatchActionResult {
        accepted: false,
        task: task_for_request(request, TaskStatus::Blocked, diagnostic.message.clone()).into(),
        diagnostics: diagnostics_from(diagnostic.message, diagnostic.code),
        device_results: request
            .device_ids
            .iter()
            .map(|device_id| AdbDeviceActionResult {
                device_id: device_id.clone(),
                success: false,
                error_kind: error_kind.into(),
                message: message.clone(),
                ..Default::default()
            })
            .collect(),
        ..BatchActionResult::default()
    }
}
