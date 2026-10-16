use std::time::Duration;

use yunheshiguang_sdk::telemetry::OperationSample;

use crate::workspace::contract::*;
use crate::workspace::text::mask_device_id;

use super::errors::batch_diagnostic_code;
use super::task::{
    action_kind, diagnostics_from, runtime_error_detail, task_for_request, task_success_detail,
};

pub(super) fn blocked_result(
    request: &BatchActionRequest,
    detail: UiText,
    code: DiagnosticCode,
) -> BatchActionResult {
    BatchActionResult {
        accepted: false,
        task: task_for_request(request, TaskStatus::Blocked, detail.clone()).into(),
        diagnostics: diagnostics_from(detail, code),
        ..BatchActionResult::default()
    }
}

pub(super) fn denied_result(request: &BatchActionRequest, reason: UiText) -> BatchActionResult {
    blocked_result(request, reason, DiagnosticCode::ActionNotAuthorized)
}

pub(super) fn audit_outcome(result: &BatchActionResult) -> AuditOutcome {
    let done = result
        .task
        .as_option()
        .is_some_and(|task| task.status == TaskStatus::Done);

    if done {
        AuditOutcome::Succeeded
    } else {
        AuditOutcome::Failed
    }
}

pub(super) fn operation_sample(
    request: &BatchActionRequest,
    result: &BatchActionResult,
    succeeded: bool,
    elapsed: Duration,
) -> OperationSample {
    let targets = u32::try_from(request.device_ids.len()).unwrap_or(u32::MAX);
    let failed = if result.device_results.is_empty() && !succeeded {
        targets
    } else {
        u32::try_from(
            result
                .device_results
                .iter()
                .filter(|device| !device.success)
                .count(),
        )
        .unwrap_or(u32::MAX)
    };

    OperationSample::new(GovernedAction::RunBatchAction, succeeded, elapsed)
        .with_batch_action(action_kind(request))
        .with_targets(targets, failed)
}

pub(super) fn with_hints(
    mut result: BatchActionResult,
    hints: Vec<DeviceHint>,
) -> BatchActionResult {
    if hints.is_empty() {
        return result;
    }

    if let Some(impact) = result
        .diagnostics
        .first()
        .and_then(|signal| signal.impact.clone().into_option())
    {
        result.diagnostics = diagnostics_from(impact, DiagnosticCode::VendorPermission);
    }
    result.device_hints = hints;
    result
}

pub(super) fn summarize(
    request: BatchActionRequest,
    device_results: Vec<AdbDeviceActionResult>,
    dispatched: bool,
) -> BatchActionResult {
    let errors = device_results
        .iter()
        .filter(|result| !result.success)
        .map(|result| format!("{}: {}", mask_device_id(&result.device_id), result.message))
        .collect::<Vec<_>>();

    if errors.is_empty() {
        return BatchActionResult {
            accepted: true,
            task: task_for_request(&request, TaskStatus::Done, task_success_detail(&request))
                .into(),
            device_results,
            ..BatchActionResult::default()
        };
    }

    let detail = runtime_error_detail(errors.join("; "));
    let code = batch_diagnostic_code(&device_results);
    let status = if dispatched {
        TaskStatus::Failed
    } else {
        TaskStatus::Blocked
    };

    BatchActionResult {
        accepted: dispatched,
        task: task_for_request(&request, status, detail.clone()).into(),
        diagnostics: diagnostics_from(detail, code),
        device_results,
        ..BatchActionResult::default()
    }
}
