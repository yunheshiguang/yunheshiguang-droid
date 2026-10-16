use crate::workspace::adb::{AdbCommandError, AdbErrorKind};
use crate::workspace::android_tools::adb_missing_text;
use crate::workspace::contract::{ConnectionOutcome, ConnectionResult};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::outcome::{connection_result, failed};

const UNREACHABLE_MARKERS: [&str; 9] = [
    "failed to resolve host",
    "failed to connect",
    "unable to connect",
    "cannot connect",
    "connection refused",
    "no route to host",
    "timed out",
    "protocol fault",
    "unable to start pairing client",
];

fn verdict_line(output: &str) -> &str {
    output
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty() && !line.starts_with("* "))
        .unwrap_or("")
}

fn unreachable(reason: &str) -> ConnectionResult {
    connection_result(
        ConnectionOutcome::Unreachable,
        "",
        ui_text_with_params(CONNECTION_UNREACHABLE_KEY, "", [("reason", reason)]),
    )
}

fn classify_failure(message: &str) -> ConnectionResult {
    let lowered = message.to_ascii_lowercase();

    if lowered.contains("failed to authenticate") {
        connection_result(
            ConnectionOutcome::Rejected,
            "",
            ui_text(CONNECTION_UNAUTHORIZED_KEY),
        )
    } else if lowered.contains("wrong password") {
        connection_result(
            ConnectionOutcome::Rejected,
            "",
            ui_text(CONNECTION_PAIRING_REJECTED_KEY),
        )
    } else if UNREACHABLE_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        unreachable(message)
    } else if message.is_empty() {
        failed("adb printed no result")
    } else {
        failed(message)
    }
}

pub fn command_failure(error: &AdbCommandError) -> ConnectionResult {
    match error.kind {
        AdbErrorKind::Timeout => unreachable(&error.message),
        AdbErrorKind::NotFound => tools_unavailable(),
        _ => classify_failure(verdict_line(&error.message)),
    }
}

pub fn tools_unavailable() -> ConnectionResult {
    connection_result(ConnectionOutcome::ToolsUnavailable, "", adb_missing_text())
}

pub fn connect_result(stdout: &str) -> ConnectionResult {
    let line = verdict_line(stdout);

    if let Some(device) = line.strip_prefix("already connected to ") {
        return connection_result(
            ConnectionOutcome::AlreadyConnected,
            device,
            ui_text_with_params(CONNECTION_ALREADY_CONNECTED_KEY, "", [("device", device)]),
        );
    }

    if let Some(device) = line.strip_prefix("connected to ") {
        return connection_result(
            ConnectionOutcome::Connected,
            device,
            ui_text_with_params(CONNECTION_CONNECTED_KEY, "", [("device", device)]),
        );
    }

    classify_failure(line)
}

pub fn pair_result(stdout: &str, endpoint: &str) -> ConnectionResult {
    let line = verdict_line(stdout);
    let line = line
        .strip_prefix("Enter pairing code:")
        .map_or(line, str::trim_start);

    if line.starts_with("Successfully paired to") {
        connection_result(
            ConnectionOutcome::Paired,
            "",
            ui_text_with_params(CONNECTION_PAIRED_KEY, "", [("endpoint", endpoint)]),
        )
    } else {
        classify_failure(line)
    }
}

pub fn disconnect_result(stdout: &str, device: &str) -> ConnectionResult {
    let line = verdict_line(stdout);

    if line.starts_with("disconnected") {
        connection_result(
            ConnectionOutcome::Disconnected,
            device,
            ui_text_with_params(CONNECTION_DISCONNECTED_KEY, "", [("device", device)]),
        )
    } else {
        classify_failure(line)
    }
}

pub fn disconnect_failure(error: &AdbCommandError, device: &str) -> ConnectionResult {
    if error.message.contains("no such device") {
        connection_result(
            ConnectionOutcome::NotConnected,
            device,
            ui_text_with_params(CONNECTION_NOT_CONNECTED_KEY, "", [("device", device)]),
        )
    } else {
        command_failure(error)
    }
}
