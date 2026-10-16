use std::collections::HashSet;
use std::time::Duration;

use tokio::sync::watch;

use crate::workspace::contract::{BatchActionResult, Flow, StepRun, StepStatus, TaskStatus};

const MAX_RECORDED_OUTPUT_CHARS: usize = 4 * 1024;

pub fn step_done(result: &BatchActionResult) -> bool {
    result
        .task
        .as_option()
        .is_some_and(|task| task.status == TaskStatus::Done)
}

pub fn pending_steps(flow: &Flow) -> Vec<StepRun> {
    flow.steps
        .iter()
        .map(|step| StepRun {
            step_id: step.id.clone(),
            kind: step.kind,
            status: StepStatus::Pending.into(),
            attempts: 0,
            result: Default::default(),
            ..Default::default()
        })
        .collect()
}

pub fn merge(
    previous: Option<BatchActionResult>,
    mut latest: BatchActionResult,
) -> BatchActionResult {
    let Some(previous) = previous else {
        return latest;
    };
    let retried = latest
        .device_results
        .iter()
        .map(|result| result.device_id.clone())
        .collect::<HashSet<_>>();

    latest.device_results.extend(
        previous
            .device_results
            .into_iter()
            .filter(|result| !retried.contains(&result.device_id)),
    );
    latest
        .device_results
        .sort_by(|left, right| left.device_id.cmp(&right.device_id));
    latest.script_outputs.extend(
        previous
            .script_outputs
            .into_iter()
            .filter(|output| !retried.contains(&output.device_id)),
    );
    latest.input_routes.extend(
        previous
            .input_routes
            .into_iter()
            .filter(|route| !retried.contains(&route.device_id)),
    );
    latest
}

pub fn failed_devices(result: &BatchActionResult) -> Vec<String> {
    result
        .device_results
        .iter()
        .filter(|device| !device.success)
        .map(|device| device.device_id.clone())
        .collect()
}

pub fn trim_outputs(mut result: BatchActionResult) -> BatchActionResult {
    for output in &mut result.script_outputs {
        let characters = output.output.chars().count();

        if characters > MAX_RECORDED_OUTPUT_CHARS {
            output.output = output
                .output
                .chars()
                .skip(characters - MAX_RECORDED_OUTPUT_CHARS)
                .collect();
            output.truncated = true;
        }
    }
    result
}

pub async fn wait(duration: Duration, stop: &mut watch::Receiver<bool>) -> bool {
    if *stop.borrow() || duration.is_zero() {
        return *stop.borrow();
    }

    let slept = tokio::select! {
        () = tokio::time::sleep(duration) => true,
        _ = stop.wait_for(|stopped| *stopped) => false,
    };

    !slept || *stop.borrow()
}
