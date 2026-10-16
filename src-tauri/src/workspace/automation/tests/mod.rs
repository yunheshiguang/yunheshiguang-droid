mod library;
mod runner;
mod schedules;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tempfile::TempDir;
use yunheshiguang_contract::yunheshiguang::automation::v1::DeviceList;
use yunheshiguang_sdk::documents::{
    Collection, Document, DocumentId, DocumentStore, FolderDocumentStore,
};

use super::flows::FLOWS_COLLECTION;
use super::history::RunHistory;
use super::runner::{DeviceSource, FlowRunner, StepExecutor, StepFuture};
use crate::workspace::contract::*;
use crate::workspace::events::WorkspaceEvents;
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::telemetry::Telemetry;

const TWO_STEPS: &str = r#"{
  "name": "Morning",
  "steps": [
    {"id": "s1", "kind": "BATCH_ACTION_KIND_INPUT_TEXT", "value": "hi"},
    {"id": "s2", "kind": "BATCH_ACTION_KIND_RUN_SCRIPT", "value": "script-1"}
  ]
}"#;

fn store() -> (TempDir, Arc<FolderDocumentStore>) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = Arc::new(FolderDocumentStore::new(directory.path()));

    (directory, store)
}

fn save_flow(store: &dyn DocumentStore, id: &str, body: &str) {
    store
        .save(
            &Collection::parse(FLOWS_COLLECTION).unwrap(),
            &Document::new(DocumentId::parse(id).unwrap(), body).unwrap(),
        )
        .unwrap();
}

#[derive(Default)]
struct RecordingEvents(Mutex<Vec<(String, serde_json::Value)>>);

impl WorkspaceEvents for RecordingEvents {
    fn emit(&self, event: &'static str, payload: serde_json::Value) {
        self.0.lock().unwrap().push((event.to_string(), payload));
    }
}

struct FixedDevices(WorkspaceSnapshot);

impl DeviceSource for FixedDevices {
    fn snapshot(&self) -> WorkspaceSnapshot {
        self.0.clone()
    }
}

fn device(id: &str, status: DeviceStatus, group_id: &str, tags: &[&str]) -> DeviceRecord {
    DeviceRecord {
        id: id.to_string(),
        status: status.into(),
        group_id: group_id.to_string(),
        labels: DeviceLabels {
            tags: tags.iter().map(ToString::to_string).collect(),
            ..DeviceLabels::default()
        }
        .into(),
        ..DeviceRecord::default()
    }
}

#[derive(Default)]
struct FakeBatch {
    failures: Mutex<HashMap<String, u32>>,
    requests: Mutex<Vec<(BatchActionRequest, Principal)>>,
}

impl FakeBatch {
    fn failing(device: &str, times: u32) -> Arc<Self> {
        let fake = Self::default();
        fake.failures
            .lock()
            .unwrap()
            .insert(device.to_string(), times);
        Arc::new(fake)
    }

    fn requests(&self) -> Vec<(BatchActionRequest, Principal)> {
        self.requests.lock().unwrap().clone()
    }
}

impl StepExecutor for Arc<FakeBatch> {
    fn execute(&self, request: BatchActionRequest, principal: Principal) -> StepFuture {
        let mut failures = self.failures.lock().unwrap();
        let device_results = request
            .device_ids
            .iter()
            .map(|id| {
                let remaining = failures.entry(id.clone()).or_default();
                let success = *remaining == 0;
                if !success {
                    *remaining -= 1;
                }
                AdbDeviceActionResult {
                    device_id: id.clone(),
                    success,
                    ..AdbDeviceActionResult::default()
                }
            })
            .collect::<Vec<_>>();
        let done = device_results.iter().all(|result| result.success);
        self.requests.lock().unwrap().push((request, principal));
        let result = BatchActionResult {
            accepted: true,
            task: TaskItem {
                status: if done {
                    TaskStatus::Done.into()
                } else {
                    TaskStatus::Failed.into()
                },
                ..TaskItem::default()
            }
            .into(),
            device_results,
            ..BatchActionResult::default()
        };

        Box::pin(async move { result })
    }
}

struct Harness {
    runner: Arc<FlowRunner>,
    events: Arc<RecordingEvents>,
    governance: Arc<Governance>,
}

fn harness(
    store: Arc<FolderDocumentStore>,
    batch: Arc<FakeBatch>,
    snapshot: WorkspaceSnapshot,
    governance: Governance,
) -> Harness {
    let events = Arc::new(RecordingEvents::default());
    let governance = Arc::new(governance);
    let runner = Arc::new(FlowRunner::new(
        Box::new(batch),
        Arc::new(FixedDevices(snapshot)),
        store,
        Arc::clone(&governance),
        Arc::new(Telemetry::assemble(Vec::new())),
        Arc::clone(&events) as Arc<dyn WorkspaceEvents>,
        Arc::new(RunHistory::in_memory()),
    ));

    Harness {
        runner,
        events,
        governance,
    }
}

fn principal() -> Principal {
    Principal::new(
        "main",
        Operator {
            id: "local:tester".to_string(),
            name: "Tester".to_string(),
            roles: vec!["owner".to_string()],
            ..Default::default()
        },
    )
}

fn devices_target(ids: &[&str]) -> RunTarget {
    RunTarget {
        target: Some(run_target::Target::Devices(Box::new(DeviceList {
            device_ids: ids.iter().map(ToString::to_string).collect(),
            ..Default::default()
        }))),
        ..Default::default()
    }
}

async fn finished(runner: &FlowRunner, id: &str) -> FlowRun {
    for _ in 0..500 {
        if let Some(run) = runner.history().get(id)
            && run.status != RunStatus::Running
        {
            return run;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("run {id} did not finish");
}
