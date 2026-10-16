use std::sync::{Arc, Mutex};
use std::time::Duration;

use yunheshiguang_sdk::telemetry::{
    OperationSample, ProbeSample, TelemetryEvent, TelemetrySink, sink_info,
};

use super::Telemetry;
use super::health::PROBE_CAPACITY;
use crate::workspace::contract::{BatchActionKind, GovernedAction, TelemetrySinkInfo};

struct Collect(Arc<Mutex<Vec<TelemetryEvent>>>);

impl TelemetrySink for Collect {
    fn describe(&self) -> TelemetrySinkInfo {
        sink_info("test.collect", "test.label", "test.description")
    }

    fn record(&self, event: &TelemetryEvent) {
        self.0.lock().unwrap().push(event.clone());
    }
}

fn batch(kind: BatchActionKind, succeeded: bool, millis: u64) -> TelemetryEvent {
    TelemetryEvent::OperationFinished(
        OperationSample::new(
            GovernedAction::RunBatchAction,
            succeeded,
            Duration::from_millis(millis),
        )
        .with_batch_action(kind)
        .with_targets(3, u32::from(!succeeded)),
    )
}

#[test]
fn summarizes_each_operation_and_batch_action_separately() {
    let telemetry = Telemetry::assemble(Vec::new());

    telemetry.record(batch(BatchActionKind::Screenshot, true, 100));
    telemetry.record(batch(BatchActionKind::Screenshot, false, 300));
    telemetry.record(batch(BatchActionKind::InstallApk, true, 2_000));
    telemetry.record(TelemetryEvent::OperationFinished(OperationSample::new(
        GovernedAction::ConnectDevice,
        true,
        Duration::from_millis(40),
    )));

    let operations = telemetry.health().operations;
    let screenshot = operations
        .iter()
        .find(|operation| operation.batch_action == BatchActionKind::Screenshot)
        .unwrap();

    assert_eq!(operations.len(), 3);
    assert_eq!((screenshot.runs, screenshot.failures), (2, 1));
    assert_eq!((screenshot.average_ms, screenshot.slowest_ms), (200, 300));
    assert!(screenshot.last_finished_at.is_set());
}

#[test]
fn keeps_the_newest_probes_first_up_to_its_capacity() {
    let telemetry = Telemetry::assemble(Vec::new());

    for online in 0..(PROBE_CAPACITY as u32 + 5) {
        telemetry.record(TelemetryEvent::DevicesProbed(
            ProbeSample::new(Duration::from_millis(20)).with_counts(online + 1, online, 1, 0),
        ));
    }

    let health = telemetry.health();

    assert_eq!(health.probes.len(), PROBE_CAPACITY);
    assert_eq!(health.probe_capacity, PROBE_CAPACITY as u32);
    assert_eq!(health.probes[0].online, PROBE_CAPACITY as u32 + 4);
    assert_eq!(health.probes[0].unauthorized, 1);
}

#[test]
fn hands_every_event_to_contributed_sinks_and_names_them() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let telemetry = Telemetry::assemble(vec![Box::new(Collect(Arc::clone(&received)))]);

    telemetry.record(batch(BatchActionKind::Screenshot, true, 10));

    assert_eq!(received.lock().unwrap().len(), 1);
    assert_eq!(
        telemetry
            .health()
            .sinks
            .iter()
            .map(|sink| sink.id.as_str())
            .collect::<Vec<_>>(),
        vec!["core.health", "test.collect"]
    );
}
