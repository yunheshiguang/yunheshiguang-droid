use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use yunheshiguang_contract::KnownEnum;

use crate::workspace::contract::*;
use crate::workspace::diagnostics::signal;
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::policy::{BATCH_CONCURRENCY, INITIAL_RETRY_COUNT};
use super::spec::spec_for;

static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn action_kind(request: &BatchActionRequest) -> BatchActionKind {
    BatchActionKind::known(request.kind).unwrap_or(BatchActionKind::Unspecified)
}

pub(super) fn task_success_detail(request: &BatchActionRequest) -> UiText {
    spec_for(action_kind(request)).success_detail.text(request)
}

pub(super) fn task_for_request(
    request: &BatchActionRequest,
    status: TaskStatus,
    detail: UiText,
) -> TaskItem {
    let kind = action_kind(request);
    let count = request.device_ids.len().to_string();

    TaskItem {
        id: task_id(kind),
        title: ui_text(spec_for(kind).title_key).into(),
        target: ui_text_with_params(TASK_TARGET_COUNT_KEY, "", [("count", count)]).into(),
        status: status.into(),
        progress: if matches!(status, TaskStatus::Done) {
            100
        } else {
            0
        },
        detail: detail.into(),
        action_kind: request.kind,
        concurrency_limit: BATCH_CONCURRENCY as u32,
        retry_count: INITIAL_RETRY_COUNT,
        ..Default::default()
    }
}

fn task_id(kind: BatchActionKind) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let sequence = TASK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("batch-{}-{timestamp}-{sequence}", spec_for(kind).slug)
}

pub(super) fn validate_request(request: &BatchActionRequest) -> Option<UiText> {
    if request.device_ids.is_empty() {
        return Some(ui_text(VALIDATION_NEED_REAL_DEVICE_KEY));
    }
    if request.device_ids.iter().collect::<HashSet<_>>().len() != request.device_ids.len() {
        return Some(runtime_error_detail(
            "duplicate device identifiers are not allowed",
        ));
    }

    spec_for(action_kind(request)).requirement.unmet(request)
}

pub(super) fn runtime_error_detail(message: impl Into<String>) -> UiText {
    ui_text_with_params(
        DIAGNOSTIC_IMPACT_RUNTIME_KEY,
        "",
        [("message", message.into())],
    )
}

pub(super) fn diagnostics_from(detail: UiText, code: DiagnosticCode) -> Vec<DiagnosticSignal> {
    vec![signal("batch-diagnostic-0", code, detail)]
}
