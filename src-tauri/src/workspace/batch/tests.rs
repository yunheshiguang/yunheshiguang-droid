use super::actions::handler;
use super::devices::plan_devices;
use super::executor::sanitize_device_error;
use super::outcome::{audit_outcome, denied_result, summarize};
use crate::workspace::adb::{AdbDevice, AdbDeviceState, AdbService};
use crate::workspace::contract::*;
use crate::workspace::text::ui_text;

fn adb_device(serial: &str, state: AdbDeviceState) -> AdbDevice {
    AdbDevice {
        serial: serial.to_string(),
        model: None,
        device: None,
        product: None,
        usb: None,
        transport_id: None,
        state,
    }
}

#[test]
fn removes_device_serials_from_runtime_errors() {
    assert_eq!(
        sanitize_device_error("error for emulator-5554", "emulator-5554"),
        "error for <device>"
    );
}

#[test]
fn plans_reconnect_for_online_and_offline_devices_independently() {
    let devices = vec![
        adb_device("online", AdbDeviceState::Online),
        adb_device("offline", AdbDeviceState::Offline),
    ];
    let requested = vec![
        "online".to_string(),
        "offline".to_string(),
        "missing".to_string(),
    ];

    let (runnable, failures) = plan_devices(
        &devices,
        &requested,
        handler(BatchActionKind::RecoverConnection).allowed_states(),
        &AdbService::default(),
    );

    assert_eq!(runnable, vec!["online", "offline"]);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].device_id, "missing");
}

fn screenshot_request() -> BatchActionRequest {
    BatchActionRequest {
        kind: BatchActionKind::Screenshot.into(),
        device_ids: vec!["R58M20ABCDE".to_string()],
        path: "/tmp/shots".to_string(),
        ..BatchActionRequest::default()
    }
}

#[test]
fn reports_a_governance_denial_as_a_blocked_task() {
    let result = denied_result(&screenshot_request(), ui_text("test.denied"));

    assert!(!result.accepted);
    assert_eq!(result.task.as_option().unwrap().status, TaskStatus::Blocked);
    assert_eq!(
        result.diagnostics[0].code,
        DiagnosticCode::ActionNotAuthorized
    );
    assert_eq!(
        result.diagnostics[0].impact.as_option().unwrap().key,
        "test.denied"
    );
}

#[test]
fn audits_only_a_fully_completed_batch_as_a_success() {
    let request = screenshot_request();
    let done = summarize(request.clone(), Vec::new(), true);
    let partial = summarize(
        request,
        vec![AdbDeviceActionResult {
            device_id: "R58M20ABCDE".to_string(),
            success: false,
            message: "device offline".to_string(),
            ..AdbDeviceActionResult::default()
        }],
        true,
    );

    assert_eq!(audit_outcome(&done), AuditOutcome::Succeeded);
    assert_eq!(audit_outcome(&partial), AuditOutcome::Failed);
}
