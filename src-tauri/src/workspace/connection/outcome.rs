use crate::workspace::contract::{ConnectionOutcome, ConnectionResult, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text_with_params;
use yunheshiguang_contract::KnownEnum;

pub fn connection_result(
    outcome: ConnectionOutcome,
    device_id: impl Into<String>,
    detail: UiText,
) -> ConnectionResult {
    ConnectionResult {
        outcome: outcome.into(),
        device_id: device_id.into(),
        detail: detail.into(),
        ..Default::default()
    }
}

pub fn invalid_request(detail: UiText) -> ConnectionResult {
    connection_result(ConnectionOutcome::InvalidRequest, "", detail)
}

pub fn failed(reason: impl Into<String>) -> ConnectionResult {
    connection_result(
        ConnectionOutcome::Failed,
        "",
        ui_text_with_params(CONNECTION_FAILED_KEY, "", [("reason", reason.into())]),
    )
}

pub fn outcome_of(result: &ConnectionResult) -> ConnectionOutcome {
    ConnectionOutcome::known(result.outcome).unwrap_or_default()
}

pub fn succeeded(result: &ConnectionResult) -> bool {
    matches!(
        outcome_of(result),
        ConnectionOutcome::Connected
            | ConnectionOutcome::AlreadyConnected
            | ConnectionOutcome::Paired
            | ConnectionOutcome::Disconnected
    )
}
