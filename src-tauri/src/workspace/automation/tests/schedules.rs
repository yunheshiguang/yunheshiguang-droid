use std::sync::Arc;

use chrono::{Duration, Local};

use super::super::history::{DETAILED_RUNS, MAX_FINISHED_RUNS, RunHistory};
use super::super::scheduler::schedule_principal;
use super::super::schedules::{ScheduleBook, check_schedule};
use super::*;
use crate::workspace::local_state::write_json;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

fn schedule(id: &str, cron: &str) -> Schedule {
    Schedule {
        id: id.to_string(),
        name: " Morning ".to_string(),
        flow_id: "morning".to_string(),
        target: devices_target(&["a"]).into(),
        cron: cron.to_string(),
        enabled: true,
        options: Default::default(),
        owner: Default::default(),
        ..Default::default()
    }
}

fn run(id: &str, status: RunStatus, schedule_id: &str) -> FlowRun {
    FlowRun {
        id: id.to_string(),
        status: status.into(),
        schedule_id: schedule_id.to_string(),
        steps: vec![StepRun {
            status: StepStatus::Pending.into(),
            ..StepRun::default()
        }],
        ..FlowRun::default()
    }
}

#[test]
fn validates_schedules_before_saving_them() {
    let checked = check_schedule(schedule("s-1", "0 9 * * 1-5")).unwrap();
    assert_eq!(checked.name, "Morning");
    assert!(check_schedule(schedule("s-2", "@daily")).is_ok());

    let invalid = |change: fn(&mut Schedule)| {
        let mut candidate = schedule("s-1", "0 9 * * *");
        change(&mut candidate);
        check_schedule(candidate).unwrap_err().key
    };

    assert_eq!(
        invalid(|s| s.id = "../x".to_string()),
        SCHEDULE_ID_INVALID_KEY
    );
    assert_eq!(
        invalid(|s| s.name = "  ".to_string()),
        SCHEDULE_NAME_REQUIRED_KEY
    );
    assert_eq!(
        invalid(|s| s.name = "x".repeat(33)),
        SCHEDULE_NAME_TOO_LONG_KEY
    );
    assert_eq!(invalid(|s| s.flow_id.clear()), SCHEDULE_FLOW_REQUIRED_KEY);
    assert_eq!(
        invalid(|s| s.target = Default::default()),
        AUTOMATION_TARGET_MISSING_KEY
    );
    assert_eq!(
        invalid(|s| s.cron = "every day".to_string()),
        SCHEDULE_CRON_INVALID_KEY
    );
    assert_eq!(
        invalid(|s| s.cron = "0 0 9 * * *".to_string()),
        SCHEDULE_CRON_INVALID_KEY
    );
}

#[test]
fn runs_a_schedule_as_the_operator_who_saved_it() {
    let governance = Governance::core();
    let mut owned = check_schedule(schedule("s-1", "0 9 * * *")).unwrap();
    owned.owner = principal().operator.into();

    let scheduled = schedule_principal(&owned, &governance);
    assert_eq!(scheduled.actor, "schedule:Morning");
    assert_eq!(scheduled.operator, principal().operator);

    owned.owner = Default::default();
    assert_eq!(
        schedule_principal(&owned, &governance).operator,
        governance.current_operator(),
        "schedules saved before owners were recorded run as this computer's operator"
    );
}

#[test]
fn fires_each_due_schedule_once_without_catching_up() {
    let book = ScheduleBook::in_memory();
    book.save(schedule("every-minute", "* * * * *")).unwrap();
    book.save(Schedule {
        enabled: false,
        ..schedule("off", "* * * * *")
    })
    .unwrap();

    let next = book
        .next_due()
        .expect("an enabled schedule has a next time");
    assert!(next > Local::now());
    assert!(book.due(&Local::now()).is_empty());

    let on_time = next + Duration::seconds(30);
    let due = book.due(&on_time);
    assert_eq!(
        due.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        ["every-minute"]
    );
    assert!(book.due(&on_time).is_empty());
    assert!(book.next_due().unwrap() > on_time);

    let after_sleep = Local::now() + Duration::hours(3);
    assert!(
        book.due(&after_sleep).is_empty(),
        "a time missed while the computer slept does not run"
    );
    assert!(book.next_due().unwrap() > after_sleep);
}

#[test]
fn keeps_one_document_per_schedule_and_reports_their_last_run() {
    let directory = tempfile::tempdir().unwrap();
    let open = || ScheduleBook::open(Arc::new(FolderDocumentStore::new(directory.path())));
    open()
        .save(check_schedule(schedule("s-1", "0 9 * * *")).unwrap())
        .unwrap();
    let file = directory.path().join("schedules/s-1.json");
    assert!(file.is_file());
    std::fs::write(
        directory.path().join("schedules/broken.json"),
        "{\"cron\": 5}",
    )
    .unwrap();

    let book = open();
    let history = RunHistory::in_memory();
    history.update(&run("0001", RunStatus::Failed, "s-1"));
    history.update(&run("0002", RunStatus::Succeeded, "s-1"));
    history.update(&run("0003", RunStatus::Succeeded, "other"));

    let states = book.states(&history).schedules;
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].last_run_id, "0002");
    assert_eq!(states[0].last_status, RunStatus::Succeeded);
    assert!(states[0].next_run_at.is_set());
    assert_eq!(states[0].problem, Default::default());

    book.record_problem("s-1", Some(ui_text(AUTOMATION_NO_DEVICES_KEY)));
    assert_eq!(
        book.states(&history).schedules[0].problem,
        ui_text(AUTOMATION_NO_DEVICES_KEY).into()
    );
    assert_eq!(
        book.remove("s-1").unwrap().map(|s| s.name),
        Some("Morning".to_string())
    );
    assert!(!file.exists());
    assert!(open().states(&history).schedules.is_empty());
}

#[test]
fn keeps_recent_history_and_marks_runs_cut_short_as_interrupted() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("runs.json");
    write_json(
        &path,
        &FlowRuns {
            runs: vec![run("0001", RunStatus::Running, "")],
            ..Default::default()
        },
    )
    .unwrap();

    let history = RunHistory::load(path.clone());
    let restored = history.get("0001").unwrap();
    assert_eq!(restored.status, RunStatus::Failed);
    assert_eq!(restored.detail, ui_text(AUTOMATION_INTERRUPTED_KEY).into());
    assert_eq!(restored.steps[0].status, StepStatus::Skipped);
    assert_eq!(
        history
            .take_interrupted()
            .iter()
            .map(|run| run.id.as_str())
            .collect::<Vec<_>>(),
        ["0001"]
    );
    assert!(history.take_interrupted().is_empty());
    assert!(
        RunHistory::load(path.clone()).take_interrupted().is_empty(),
        "a run reported as cut short is not reported again on the next start"
    );

    history.update(&run("9999", RunStatus::Running, ""));
    for index in 0..MAX_FINISHED_RUNS + 5 {
        history.update(&run(&format!("1{index:04}"), RunStatus::Succeeded, ""));
    }
    let runs = history.list();
    assert_eq!(runs.len(), MAX_FINISHED_RUNS + 1);
    assert!(runs.iter().any(|run| run.id == "9999"));
    assert!(runs.windows(2).all(|pair| pair[0].id > pair[1].id));

    history.persist();
    let reloaded = RunHistory::load(path);
    assert_eq!(reloaded.list().len(), MAX_FINISHED_RUNS);
    assert_eq!(
        reloaded.get("9999").unwrap().detail,
        ui_text(AUTOMATION_INTERRUPTED_KEY).into()
    );
}

#[test]
fn keeps_step_details_for_recent_runs_and_settles_running_ones_on_quit() {
    let history = RunHistory::in_memory();
    let detailed = |id: &str, status: RunStatus| FlowRun {
        steps: vec![StepRun {
            status: StepStatus::Done.into(),
            result: BatchActionResult::default().into(),
            ..StepRun::default()
        }],
        ..run(id, status, "")
    };

    history.update(&detailed("9999", RunStatus::Running));
    for index in 0..DETAILED_RUNS + 3 {
        history.update(&detailed(&format!("1{index:04}"), RunStatus::Succeeded));
    }
    let runs = history.list();
    let with_details = runs
        .iter()
        .filter(|run| run.steps[0].result.is_set())
        .count();
    assert_eq!(
        with_details,
        DETAILED_RUNS + 1,
        "the running one keeps its details"
    );

    let interrupted = history.interrupt_running();
    assert_eq!(interrupted.len(), 1);
    assert_eq!(
        history.get("9999").unwrap().detail,
        ui_text(AUTOMATION_INTERRUPTED_KEY).into()
    );
    assert!(history.interrupt_running().is_empty());
}
