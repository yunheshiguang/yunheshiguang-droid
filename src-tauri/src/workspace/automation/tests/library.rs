use super::super::flows::{host_files_refusal, list_flows, load_flow, parse_flow};
use super::super::options::{MAX_REPEAT, MAX_RETRY_ATTEMPTS, check_options};
use super::super::targets::{check_target, resolve_target};
use super::*;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

#[test]
fn reads_flows_in_the_contract_format_and_the_earlier_frontend_format() {
    let contract = parse_flow(TWO_STEPS).expect("contract JSON should parse");
    assert_eq!(contract.name, "Morning");
    assert_eq!(contract.steps[1].kind, BatchActionKind::RunScript);

    let earlier = parse_flow(
        r#"{"version":1,"name":"Old","steps":[{"id":"a","kind":"LAUNCH_APP","value":"com.x.y"}]}"#,
    )
    .expect("the earlier format should still parse");
    assert_eq!(earlier.steps[0].kind, BatchActionKind::LaunchApp);
    assert_eq!(earlier.steps[0].value, "com.x.y");

    assert_eq!(parse_flow("[]"), None);
    let newer = parse_flow(r#"{"steps":[{"kind":"NOT_A_KIND"}]}"#)
        .expect("a kind a newer version added still reads");
    assert_eq!(newer.steps[0].kind, BatchActionKind::Unspecified);
}

#[test]
fn flags_flows_that_read_or_write_files_on_this_computer() {
    let reads = r#"{"name":"Leak","steps":[{"id":"a","kind":"UPLOAD_FILE","value":"/home/me/.ssh/id_ed25519"}]}"#;
    let writes = r#"{"name":"Shots","steps":[{"id":"a","kind":"BATCH_ACTION_KIND_SCREENSHOT","path":"/tmp/x.png"}]}"#;

    assert_eq!(
        host_files_refusal(reads).map(|reason| reason.key),
        Some(API_HOST_FILES_KEY.to_string())
    );
    assert!(host_files_refusal(writes).is_some());
    assert_eq!(host_files_refusal(TWO_STEPS), None);
    assert_eq!(host_files_refusal("not json"), None);
}

#[test]
fn lists_readable_flows_and_refuses_ones_that_cannot_run() {
    let (_directory, store) = store();
    save_flow(store.as_ref(), "morning", TWO_STEPS);
    save_flow(store.as_ref(), "empty", r#"{"name":"Empty","steps":[]}"#);
    save_flow(store.as_ref(), "broken", r#"{"name":1}"#);

    let listed = list_flows(store.as_ref()).unwrap();
    let ids = listed
        .flows
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["empty", "morning"]);

    assert!(load_flow(store.as_ref(), "morning").is_ok());
    assert_eq!(
        load_flow(store.as_ref(), "empty"),
        Err(ui_text(AUTOMATION_FLOW_EMPTY_KEY))
    );
    assert_eq!(
        load_flow(store.as_ref(), "broken"),
        Err(ui_text(AUTOMATION_FLOW_UNREADABLE_KEY))
    );
    assert_eq!(
        load_flow(store.as_ref(), "missing"),
        Err(ui_text(AUTOMATION_FLOW_MISSING_KEY))
    );
}

#[test]
fn runs_at_least_once_and_caps_repeats_retries_and_pauses() {
    let plan = check_options(None).unwrap();
    assert_eq!(plan.iterations, 1);
    assert_eq!(plan.retries, 0);

    let plan = check_options(Some(&RunOptions {
        repeat: 3,
        repeat_interval_seconds: 5,
        retry_attempts: 2,
        retry_delay_seconds: 1,
        ..Default::default()
    }))
    .unwrap();
    assert_eq!(plan.iterations, 3);
    assert_eq!(plan.interval.as_secs(), 5);

    for options in [
        RunOptions {
            repeat: MAX_REPEAT + 1,
            ..RunOptions::default()
        },
        RunOptions {
            retry_attempts: MAX_RETRY_ATTEMPTS + 1,
            ..RunOptions::default()
        },
    ] {
        assert_eq!(
            check_options(Some(&options)),
            Err(ui_text(AUTOMATION_OPTIONS_TOO_LARGE_KEY))
        );
    }
}

#[test]
fn resolves_groups_and_tags_to_their_online_members_only() {
    let snapshot = WorkspaceSnapshot {
        devices: vec![
            device("a", DeviceStatus::Online, "shelf", &["Morning"]),
            device("b", DeviceStatus::Offline, "shelf", &["morning"]),
            device("c", DeviceStatus::Idle, "ungrouped", &["night"]),
        ],
        ..WorkspaceSnapshot::default()
    };
    let resolve_with = |target: RunTarget, offline: bool| {
        let target = check_target(Some(&target))?.clone();
        resolve_target(&target, offline, || snapshot.clone())
    };
    let resolve = |target: RunTarget| resolve_with(target, false);
    let group = |id: &str| RunTarget {
        target: Some(run_target::Target::GroupId(id.to_string())),
        ..Default::default()
    };
    let tag = |tag: &str| RunTarget {
        target: Some(run_target::Target::Tag(tag.to_string())),
        ..Default::default()
    };

    assert_eq!(resolve(group("shelf")), Ok(vec!["a".to_string()]));
    assert_eq!(
        resolve_with(group("shelf"), true),
        Ok(vec!["a".to_string(), "b".to_string()]),
        "steps that recover connections reach offline members too"
    );
    assert_eq!(resolve(group("ungrouped")), Ok(vec!["c".to_string()]));
    assert_eq!(resolve(tag(" MORNING ")), Ok(vec!["a".to_string()]));
    assert_eq!(
        resolve(tag("absent")),
        Err(ui_text(AUTOMATION_NO_DEVICES_KEY))
    );
    assert_eq!(
        resolve(devices_target(&["b", "b", " ", "z"])),
        Ok(vec!["b".to_string(), "z".to_string()])
    );
    assert_eq!(
        resolve(devices_target(&[])),
        Err(ui_text(AUTOMATION_TARGET_MISSING_KEY))
    );
    assert_eq!(
        check_target(None).err(),
        Some(ui_text(AUTOMATION_TARGET_MISSING_KEY))
    );
}

#[test]
fn resolves_explicit_devices_without_probing() {
    let target = check_target(Some(&devices_target(&["x"]))).unwrap().clone();

    assert_eq!(
        resolve_target(&target, false, || {
            panic!("explicit devices need no snapshot")
        }),
        Ok(vec!["x".to_string()])
    );
}
