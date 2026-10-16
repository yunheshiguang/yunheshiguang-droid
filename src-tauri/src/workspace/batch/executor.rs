use std::collections::HashMap;
use std::path::Path;
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_sdk::input::InputRequest;
use yunheshiguang_sdk::scripts::ScriptRequest;
use yunheshiguang_sdk::vendor::DeviceIdentity;

use crate::workspace::adb::{AdbClient, AdbCommandError, AdbDeviceShell, run_bounded};
use crate::workspace::contract::*;
use crate::workspace::input::{Delivery, InputRouter, plain_message};
use crate::workspace::scripts::{ScriptRun, ScriptRunner};

use super::errors::adb_error_kind;
use super::policy::BATCH_CONCURRENCY;

pub(super) fn each_device(
    adb_path: &Path,
    serials: Vec<String>,
    run: impl Fn(&AdbClient, &str) -> Result<(), AdbCommandError> + Sync,
) -> Vec<AdbDeviceActionResult> {
    let mut results = run_bounded(
        serials,
        BATCH_CONCURRENCY,
        || AdbClient::new(adb_path),
        |client, serial| {
            let outcome = run(client, &serial);
            device_result(serial, outcome)
        },
    );
    results.sort_by(|left, right| left.device_id.cmp(&right.device_id));
    results
}

pub(super) fn deliver_concurrently(
    adb_path: &Path,
    serials: Vec<String>,
    request: InputRequest<'_>,
    router: &InputRouter,
    identities: &HashMap<String, DeviceIdentity>,
) -> (Vec<AdbDeviceActionResult>, Vec<DeviceInputRoute>) {
    let mut outcomes = run_bounded(
        serials,
        BATCH_CONCURRENCY,
        || AdbClient::new(adb_path),
        |client, serial| {
            let shell = AdbDeviceShell::new(client, &serial, identities.get(&serial));
            let delivery = router.deliver(&shell, &request);

            (serial, delivery)
        },
    );
    outcomes.sort_by(|left, right| left.0.cmp(&right.0));
    outcomes
        .into_iter()
        .map(|(serial, delivery)| delivered(serial, delivery))
        .unzip()
}

fn delivered(serial: String, delivery: Delivery) -> (AdbDeviceActionResult, DeviceInputRoute) {
    let Delivery { route, result } = delivery;
    let (error_kind, message) = match &result {
        Ok(()) => (AdbErrorKindContract::Unspecified, String::new()),
        Err(failure) if route.is_none() => {
            (AdbErrorKindContract::Unsupported, plain_message(failure))
        }
        Err(failure) => (
            AdbErrorKindContract::NonZeroExit,
            sanitize_device_error(&plain_message(failure), &serial),
        ),
    };
    let (channel_id, channel) = route
        .map(|route| (route.id, route.label))
        .unwrap_or_default();

    (
        AdbDeviceActionResult {
            device_id: serial.clone(),
            success: result.is_ok(),
            error_kind: error_kind.into(),
            message,
            ..Default::default()
        },
        DeviceInputRoute {
            device_id: serial,
            channel_id,
            channel: channel.into(),
            failure: result.err().into(),
            ..Default::default()
        },
    )
}

pub(super) fn run_scripts_concurrently(
    adb_path: &Path,
    serials: Vec<String>,
    script_id: &str,
    script: &Script,
    runner: &ScriptRunner,
    identities: &HashMap<String, DeviceIdentity>,
) -> (Vec<AdbDeviceActionResult>, Vec<DeviceScriptOutput>) {
    let language = ScriptLanguage::known(script.language).unwrap_or(ScriptLanguage::Unspecified);
    let request = ScriptRequest::new(script_id, &script.name, language, &script.body);
    let mut outcomes = run_bounded(
        serials,
        BATCH_CONCURRENCY,
        || AdbClient::new(adb_path),
        |client, serial| {
            let shell = AdbDeviceShell::new(client, &serial, identities.get(&serial));
            let run = runner.run(&shell, &request);

            (serial, run)
        },
    );
    outcomes.sort_by(|left, right| left.0.cmp(&right.0));
    outcomes
        .into_iter()
        .map(|(serial, run)| script_finished(serial, run))
        .unzip()
}

fn script_finished(serial: String, run: ScriptRun) -> (AdbDeviceActionResult, DeviceScriptOutput) {
    let ScriptRun { engine, result } = run;
    let (engine_id, engine_label) = engine
        .map(|engine| (engine.id, engine.label))
        .unwrap_or_default();
    let (error_kind, message, output) = match result {
        Ok(outcome) => {
            let failed = outcome.exit_code.filter(|code| *code != 0);
            let error_kind = if failed.is_some() {
                AdbErrorKindContract::NonZeroExit
            } else {
                AdbErrorKindContract::Unspecified
            };
            let message = failed
                .map(|code| format!("script exited with status {code}"))
                .unwrap_or_default();

            (error_kind, message, Ok(outcome))
        }
        Err(failure) if engine_id.is_empty() => (
            AdbErrorKindContract::Unsupported,
            plain_message(&failure),
            Err(failure),
        ),
        Err(failure) => (
            AdbErrorKindContract::NonZeroExit,
            sanitize_device_error(&plain_message(&failure), &serial),
            Err(failure),
        ),
    };
    let success = error_kind == AdbErrorKindContract::Unspecified;
    let (exit_code, text, truncated, failure) = match output {
        Ok(outcome) => (
            outcome.exit_code,
            sanitize_device_error(&outcome.output, &serial),
            outcome.truncated,
            None,
        ),
        Err(failure) => (None, String::new(), false, Some(failure)),
    };

    (
        AdbDeviceActionResult {
            device_id: serial.clone(),
            success,
            error_kind: error_kind.into(),
            message,
            ..Default::default()
        },
        DeviceScriptOutput {
            device_id: serial,
            engine_id,
            engine: engine_label.into(),
            exit_code,
            output: text,
            truncated,
            failure: failure.into(),
            ..Default::default()
        },
    )
}

fn device_result(serial: String, outcome: Result<(), AdbCommandError>) -> AdbDeviceActionResult {
    match outcome {
        Ok(()) => AdbDeviceActionResult {
            device_id: serial,
            success: true,
            error_kind: AdbErrorKindContract::Unspecified.into(),
            message: String::new(),
            ..Default::default()
        },
        Err(error) => AdbDeviceActionResult {
            message: sanitize_device_error(&error.to_string(), &serial),
            device_id: serial,
            success: false,
            error_kind: adb_error_kind(error.kind).into(),
            ..Default::default()
        },
    }
}

pub(super) fn sanitize_device_error(message: &str, serial: &str) -> String {
    message.replace(serial, "<device>")
}
