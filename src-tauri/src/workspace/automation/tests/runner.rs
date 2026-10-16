use std::sync::Arc;

use yunheshiguang_sdk::governance::{ActionContext, AuthorizationProvider, Decision};

use super::super::runner::{AUTOMATION_CHANGED_EVENT, StartRun};
use super::*;
use crate::workspace::governance::GovernanceParts;
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

fn start(flow_id: &str, target: RunTarget, options: RunOptions) -> StartRun {
    StartRun {
        flow_id: flow_id.to_string(),
        target: Some(target),
        options: Some(options),
        trigger: RunTrigger::Manual,
        schedule_id: String::new(),
    }
}

fn audited(governance: &Governance, action: GovernedAction) -> Vec<AuditEntry> {
    governance
        .state()
        .audit_trail
        .into_iter()
        .filter(|entry| entry.action == action)
        .collect()
}

#[test]
fn runs_every_step_on_every_repetition() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    let batch = Arc::new(FakeBatch::default());
    let harness = harness(
        store,
        Arc::clone(&batch),
        WorkspaceSnapshot::default(),
        Governance::core(),
    );

    let run = tauri::async_runtime::block_on(async {
        let started = harness
            .runner
            .start(
                start(
                    "morning",
                    devices_target(&["a", "b"]),
                    RunOptions {
                        repeat: 2,
                        ..RunOptions::default()
                    },
                ),
                &principal(),
            )
            .await;
        assert!(started.accepted, "{:?}", started.detail);
        finished(&harness.runner, &started.run.unwrap().id).await
    });

    assert_eq!(run.status, RunStatus::Succeeded);
    assert_eq!((run.iteration, run.iterations), (2, 2));
    assert_eq!(run.flow_name, "Morning");
    assert!(
        run.steps
            .iter()
            .all(|step| step.status == StepStatus::Done && step.attempts == 1)
    );

    assert_eq!(run.operator, principal().operator.into());
    assert_eq!(
        (
            run.target.into_option(),
            run.options.map(|options| options.repeat)
        ),
        (Some(devices_target(&["a", "b"])), Some(2)),
        "a finished run keeps what it needs to run again"
    );

    let requests = batch.requests();
    assert_eq!(requests.len(), 4);
    let (typing, step_principal) = &requests[0];
    assert_eq!(typing.kind, BatchActionKind::InputText);
    assert_eq!(typing.text, "hi");
    assert_eq!(typing.device_ids, ["a", "b"]);
    assert_eq!(step_principal, &principal());
    assert_eq!(requests[1].0.kind, BatchActionKind::RunScript);
    assert_eq!(requests[1].0.script_id, "script-1");

    let events = harness.events.0.lock().unwrap();
    assert!(events.len() >= 6);
    assert!(
        events
            .iter()
            .all(|(event, payload)| event == AUTOMATION_CHANGED_EVENT
                && payload["runId"] == run.id.as_str())
    );

    let entries = audited(&harness.governance, GovernedAction::RunAutomation);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].outcome, AuditOutcome::Succeeded);
    assert_eq!(entries[0].actor, "main");
    assert_eq!(entries[0].operator, principal().operator.into());
}

#[test]
fn retries_only_the_devices_that_failed() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    let batch = FakeBatch::failing("b", 1);
    let harness = harness(
        store,
        Arc::clone(&batch),
        WorkspaceSnapshot::default(),
        Governance::core(),
    );

    let run = tauri::async_runtime::block_on(async {
        let started = harness
            .runner
            .start(
                start(
                    "morning",
                    devices_target(&["a", "b"]),
                    RunOptions {
                        retry_attempts: 2,
                        ..RunOptions::default()
                    },
                ),
                &principal(),
            )
            .await;
        finished(&harness.runner, &started.run.unwrap().id).await
    });

    assert_eq!(run.status, RunStatus::Succeeded);
    let first = &run.steps[0];
    assert_eq!(first.attempts, 2);
    let results = &first.result.as_option().unwrap().device_results;
    assert_eq!(
        results
            .iter()
            .map(|result| (result.device_id.as_str(), result.success))
            .collect::<Vec<_>>(),
        [("a", true), ("b", true)]
    );
    assert_eq!(batch.requests()[1].0.device_ids, ["b"]);
}

#[test]
fn shows_each_failed_attempt_while_the_step_waits_to_retry() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    let harness = harness(
        store,
        FakeBatch::failing("b", 1),
        WorkspaceSnapshot::default(),
        Governance::core(),
    );

    tauri::async_runtime::block_on(async {
        let started = harness
            .runner
            .start(
                start(
                    "morning",
                    devices_target(&["a", "b"]),
                    RunOptions {
                        retry_attempts: 1,
                        retry_delay_seconds: 1,
                        ..RunOptions::default()
                    },
                ),
                &principal(),
            )
            .await;
        let id = started.run.unwrap().id;
        let mut waiting = None;
        for _ in 0..100 {
            let run = harness.runner.history().get(&id).unwrap();
            if run.status == RunStatus::Running && run.steps[0].attempts == 1 {
                waiting = Some(run);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let waiting = waiting.expect("the failed attempt shows before the retry");
        assert_eq!(waiting.steps[0].status, StepStatus::Running);
        assert!(
            waiting.steps[0]
                .result
                .as_option()
                .unwrap()
                .device_results
                .iter()
                .any(|result| !result.success)
        );
        assert_eq!(
            finished(&harness.runner, &id).await.status,
            RunStatus::Succeeded
        );
    });
}

#[test]
fn fails_the_run_at_the_first_step_that_keeps_failing() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    let batch = FakeBatch::failing("a", 5);
    let harness = harness(
        store,
        Arc::clone(&batch),
        WorkspaceSnapshot::default(),
        Governance::core(),
    );

    let run = tauri::async_runtime::block_on(async {
        let started = harness
            .runner
            .start(
                start(
                    "morning",
                    devices_target(&["a"]),
                    RunOptions {
                        retry_attempts: 1,
                        repeat: 3,
                        ..RunOptions::default()
                    },
                ),
                &principal(),
            )
            .await;
        finished(&harness.runner, &started.run.unwrap().id).await
    });

    assert_eq!(run.status, RunStatus::Failed);
    assert_eq!(run.iteration, 1);
    assert_eq!(
        run.detail,
        ui_text_with_params(AUTOMATION_STEP_FAILED_KEY, "", [("step", "1".to_string())]).into()
    );
    assert_eq!(run.steps[0].status, StepStatus::Failed);
    assert_eq!(run.steps[0].attempts, 2);
    assert_eq!(run.steps[1].status, StepStatus::Skipped);
    assert_eq!(batch.requests().len(), 2);
    assert_eq!(
        audited(&harness.governance, GovernedAction::RunAutomation)[0].outcome,
        AuditOutcome::Failed
    );
}

#[test]
fn stops_between_repetitions_when_asked() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    let harness = harness(
        store,
        Arc::new(FakeBatch::default()),
        WorkspaceSnapshot::default(),
        Governance::core(),
    );

    let run = tauri::async_runtime::block_on(async {
        let started = harness
            .runner
            .start(
                start(
                    "morning",
                    devices_target(&["a"]),
                    RunOptions {
                        repeat: 2,
                        repeat_interval_seconds: 3600,
                        ..RunOptions::default()
                    },
                ),
                &principal(),
            )
            .await;
        let id = started.run.unwrap().id;
        for _ in 0..500 {
            let current = harness.runner.history().get(&id).unwrap();
            if current.steps[1].status == StepStatus::Done {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(harness.runner.stop(&id, &principal()).accepted);
        finished(&harness.runner, &id).await
    });

    assert_eq!(run.status, RunStatus::Stopped);
    assert_eq!(run.iteration, 1);
    assert_eq!(run.detail, ui_text(AUTOMATION_STOPPED_KEY).into());
    assert_eq!(
        harness.runner.stop(&run.id, &principal()).detail,
        ui_text(AUTOMATION_RUN_NOT_RUNNING_KEY).into()
    );
}

struct DenyAll;

impl AuthorizationProvider for DenyAll {
    fn describe(&self) -> GovernanceProvider {
        GovernanceProvider::default()
    }

    fn authorize(&self, _context: &ActionContext<'_>) -> Decision {
        Decision::Deny(ui_text("test.denied"))
    }
}

#[test]
fn refuses_runs_that_cannot_start_and_records_none_of_them() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    let offline = WorkspaceSnapshot {
        devices: vec![device("a", DeviceStatus::Offline, "shelf", &[])],
        ..WorkspaceSnapshot::default()
    };
    let open = harness(
        Arc::clone(&store),
        Arc::new(FakeBatch::default()),
        offline,
        Governance::core(),
    );
    let denied = harness(
        store,
        Arc::new(FakeBatch::default()),
        WorkspaceSnapshot::default(),
        Governance::assemble(GovernanceParts {
            authorization: Some(Box::new(DenyAll)),
            ..GovernanceParts::default()
        }),
    );
    let shelf = RunTarget {
        target: Some(run_target::Target::GroupId("shelf".to_string())),
        ..Default::default()
    };
    let refusal = |harness: &Harness, start: StartRun| {
        tauri::async_runtime::block_on(harness.runner.start(start, &principal())).detail
    };

    assert_eq!(
        refusal(
            &open,
            start("absent", devices_target(&["a"]), RunOptions::default())
        ),
        ui_text(AUTOMATION_FLOW_MISSING_KEY).into()
    );
    assert_eq!(
        refusal(&open, start("morning", shelf, RunOptions::default())),
        ui_text(AUTOMATION_NO_DEVICES_KEY).into()
    );
    assert_eq!(
        refusal(
            &open,
            start(
                "morning",
                devices_target(&["a"]),
                RunOptions {
                    repeat: 5000,
                    ..RunOptions::default()
                }
            )
        ),
        ui_text(AUTOMATION_OPTIONS_TOO_LARGE_KEY).into()
    );
    assert_eq!(
        refusal(
            &denied,
            start("morning", devices_target(&["a"]), RunOptions::default())
        ),
        ui_text("test.denied").into()
    );
    assert_eq!(
        audited(&denied.governance, GovernedAction::RunAutomation)[0].outcome,
        AuditOutcome::Denied
    );
    assert!(open.runner.runs().runs.is_empty());
    assert!(denied.runner.runs().runs.is_empty());
}

#[test]
fn refuses_and_audits_local_api_runs_of_flows_that_touch_files_on_this_computer() {
    let (_directory, store) = store();
    save_flow(
        store.as_ref(),
        "upload",
        r#"{"name":"Upload","steps":[{"id":"a","kind":"UPLOAD_FILE","value":"/sdcard/x"}]}"#,
    );
    let harness = harness(
        store,
        Arc::new(FakeBatch::default()),
        WorkspaceSnapshot::default(),
        Governance::core(),
    );
    let api = Principal::new("api:robot", principal().operator);

    let result = tauri::async_runtime::block_on(harness.runner.start(
        start("upload", devices_target(&["a"]), RunOptions::default()),
        &api,
    ));

    assert!(!result.accepted);
    assert_eq!(result.detail, ui_text(API_HOST_FILES_KEY).into());
    assert_eq!(
        audited(&harness.governance, GovernedAction::RunAutomation)[0].outcome,
        AuditOutcome::Denied
    );
    assert!(harness.runner.runs().runs.is_empty());
}
