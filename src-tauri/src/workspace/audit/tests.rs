use std::path::Path;
use std::sync::Arc;

use tempfile::TempDir;
use yunheshiguang_contract::wkt::Timestamp;
use yunheshiguang_sdk::governance::{ActionContext, AuditStore, AuthorizationProvider, Decision};

use super::query::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, page_size};
use super::requests::export_as;
use super::sqlite::SqliteAuditStore;
use crate::workspace::contract::*;
use crate::workspace::governance::{Governance, GovernanceParts, Principal};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

fn at(seconds: i64) -> Timestamp {
    Timestamp {
        seconds,
        nanos: 0,
        ..Default::default()
    }
}

fn operator(id: &str) -> Operator {
    Operator {
        id: id.to_string(),
        name: id.to_string(),
        roles: Vec::new(),
        ..Default::default()
    }
}

fn entry(
    id: &str,
    seconds: i64,
    action: GovernedAction,
    outcome: AuditOutcome,
    operator_id: &str,
    actor: &str,
) -> AuditEntry {
    AuditEntry {
        id: id.to_string(),
        occurred_at: at(seconds).into(),
        action: action.into(),
        outcome: outcome.into(),
        actor: actor.to_string(),
        operator: operator(operator_id).into(),
        groups: vec!["shelf-a".to_string()],
        ..AuditEntry::default()
    }
}

fn simple(id: &str, seconds: i64) -> AuditEntry {
    entry(
        id,
        seconds,
        GovernedAction::PairDevice,
        AuditOutcome::Succeeded,
        "local:alice",
        "main",
    )
}

fn open_in(directory: &Path) -> SqliteAuditStore {
    SqliteAuditStore::open(&directory.join("audit").join("audit.sqlite3")).unwrap()
}

fn ids(store: &SqliteAuditStore, query: AuditQuery) -> Vec<String> {
    store
        .query(&query)
        .unwrap()
        .entries
        .into_iter()
        .map(|entry| entry.id)
        .collect()
}

#[test]
fn keeps_whole_entries_newest_first_across_restarts() {
    let directory = tempfile::tempdir().unwrap();
    {
        let store = open_in(directory.path());
        store.append(&simple("a", 100));
        store.append(&simple("b", 200));
        store.flush();
    }

    let store = open_in(directory.path());
    let page = store.query(&AuditQuery::default()).unwrap();

    assert_eq!(page.entries, vec![simple("b", 200), simple("a", 100)]);
    assert!(page.next_before_id.is_empty());
    assert!(page.problem.is_unset());
}

#[test]
fn filters_by_action_outcome_operator_actor_and_time() {
    use AuditOutcome::{
        AUDIT_OUTCOME_DENIED as Denied, AUDIT_OUTCOME_FAILED as Failed,
        AUDIT_OUTCOME_SUCCEEDED as Succeeded,
    };
    use GovernedAction::{
        GOVERNED_ACTION_CONNECT_DEVICE as ConnectDevice,
        GOVERNED_ACTION_RUN_AUTOMATION as RunAutomation,
        GOVERNED_ACTION_RUN_BATCH_ACTION as RunBatchAction,
    };

    let directory = tempfile::tempdir().unwrap();
    let store = open_in(directory.path());
    for entry in [
        entry("1", 100, RunBatchAction, Succeeded, "local:alice", "main"),
        entry("2", 200, RunBatchAction, Failed, "local:bob", "main"),
        entry("3", 300, ConnectDevice, Succeeded, "local:alice", "api:ci"),
        entry("4", 400, RunAutomation, Denied, "api:ci", "api:ci"),
    ] {
        store.append(&entry);
    }
    let query = |query: AuditQuery| ids(&store, query);

    assert_eq!(
        query(AuditQuery {
            actions: vec![RunBatchAction.into()],
            ..AuditQuery::default()
        }),
        ["2", "1"]
    );
    assert_eq!(
        query(AuditQuery {
            outcomes: vec![Succeeded.into(), Denied.into()],
            ..AuditQuery::default()
        }),
        ["4", "3", "1"]
    );
    assert_eq!(
        query(AuditQuery {
            operator_id: "local:alice".to_string(),
            ..AuditQuery::default()
        }),
        ["3", "1"]
    );
    assert_eq!(
        query(AuditQuery {
            actor: "api:ci".to_string(),
            ..AuditQuery::default()
        }),
        ["4", "3"]
    );
    assert_eq!(
        query(AuditQuery {
            actor: "api".to_string(),
            ..AuditQuery::default()
        }),
        ["4", "3"],
        "a channel matches every actor on it"
    );
    assert!(
        query(AuditQuery {
            actor: "ap".to_string(),
            ..AuditQuery::default()
        })
        .is_empty()
    );
    assert_eq!(
        query(AuditQuery {
            since: at(200).into(),
            until: at(400).into(),
            ..AuditQuery::default()
        }),
        ["3", "2"],
        "since is inclusive, until is exclusive"
    );
    assert!(
        query(AuditQuery {
            actions: vec![ConnectDevice.into()],
            operator_id: "local:bob".to_string(),
            ..AuditQuery::default()
        })
        .is_empty(),
        "conditions combine"
    );
}

#[test]
fn pages_backwards_from_the_newest_entry() {
    let directory = tempfile::tempdir().unwrap();
    let store = open_in(directory.path());
    for (seconds, id) in (1..).zip(["1", "2", "3", "4", "5"]) {
        store.append(&simple(id, seconds));
    }
    let page = |before_id: &str| {
        store
            .query(&AuditQuery {
                limit: 2,
                before_id: before_id.to_string(),
                ..AuditQuery::default()
            })
            .unwrap()
    };

    let first = page("");
    assert_eq!(first.next_before_id, "4");
    let second = page(&first.next_before_id);
    assert_eq!(second.next_before_id, "2");
    let last = page(&second.next_before_id);
    let ids = |page: &AuditPage| {
        page.entries
            .iter()
            .map(|e| e.id.clone())
            .collect::<Vec<_>>()
    };

    assert_eq!(
        [ids(&first), ids(&second), ids(&last)],
        [vec!["5", "4"], vec!["3", "2"], vec!["1"]]
    );
    assert!(last.next_before_id.is_empty());
    assert_eq!(page_size(0), DEFAULT_PAGE_SIZE);
    assert_eq!(page_size(u32::MAX), MAX_PAGE_SIZE);
}

#[test]
fn prunes_the_oldest_entries_beyond_the_retention() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("audit.sqlite3");
    {
        let store = SqliteAuditStore::open_retaining(&path, 3).unwrap();
        for id in ["1", "2", "3", "4", "5"] {
            store.append(&simple(id, 1));
        }
        store.flush();
    }

    let store = SqliteAuditStore::open_retaining(&path, 3).unwrap();
    assert_eq!(ids(&store, AuditQuery::default()), ["5", "4", "3"]);
}

fn governance_with_store() -> (TempDir, Arc<Governance>) {
    let directory = tempfile::tempdir().unwrap();
    let governance = Arc::new(Governance::core());
    assert!(governance.install_audit_store(Arc::new(open_in(directory.path()))));

    (directory, governance)
}

#[test]
fn stores_every_governed_action_with_its_operator() {
    let (directory, governance) = governance_with_store();
    let principal = governance.principal("main");

    governance
        .begin(GovernedAction::PairDevice, &principal, &[])
        .unwrap()
        .finish(AuditOutcome::Succeeded, None);

    let page = governance.query_audit(&AuditQuery::default());
    assert!(page.problem.is_unset());
    assert_eq!(page.entries.len(), 1);
    assert_eq!(page.entries[0].operator, principal.operator.into());
    assert_eq!(
        governance.state().audit_store.map(|store| store.id),
        Some("core.sqlite-audit".to_string())
    );
    assert!(
        !governance.install_audit_store(Arc::new(open_in(&directory.path().join("other")))),
        "the store installed first stays"
    );
}

#[test]
fn reports_the_history_as_unavailable_without_a_store() {
    let page = Governance::core().query_audit(&AuditQuery::default());

    assert_eq!(page.problem, ui_text(AUDIT_UNAVAILABLE_KEY).into());
    assert!(page.entries.is_empty());
}

fn export(
    governance: &Arc<Governance>,
    format: AuditExportFormat,
    path: &Path,
    query: AuditQuery,
) -> ExportAuditResult {
    let request = ExportAuditRequest {
        query: query.into(),
        format: format.into(),
        path: path.display().to_string(),
        scope: Default::default(),
        ..Default::default()
    };

    tauri::async_runtime::block_on(export_as(
        governance,
        request,
        &governance.principal("main"),
    ))
}

#[test]
fn exports_the_filtered_history_as_csv_and_json_lines() {
    let (directory, governance) = governance_with_store();
    let spreadsheet_trap = Principal::new("main", operator("=cmd"));
    governance
        .begin(
            GovernedAction::RunBatchAction,
            &spreadsheet_trap,
            &["R58M20ABCDE".to_string()],
        )
        .unwrap()
        .finish(
            AuditOutcome::Succeeded,
            Some(ui_text_with_params(
                "test.done",
                "",
                [("count", "2".to_string())],
            )),
        );
    governance
        .begin(GovernedAction::PairDevice, &spreadsheet_trap, &[])
        .unwrap()
        .finish(AuditOutcome::Failed, None);

    let csv_path = directory.path().join("audit.csv");
    let only_batches = AuditQuery {
        actions: vec![GovernedAction::RunBatchAction.into()],
        ..AuditQuery::default()
    };
    let result = export(&governance, AuditExportFormat::Csv, &csv_path, only_batches);
    assert!(result.accepted, "{:?}", result.detail);
    assert_eq!(result.exported, 1);

    let csv = std::fs::read_to_string(&csv_path).unwrap();
    let mut lines = csv
        .strip_prefix('\u{feff}')
        .expect("Excel needs the BOM to read UTF-8")
        .lines();
    assert_eq!(
        lines.next(),
        Some(
            "id,occurred_at,action,outcome,actor,operator_id,operator_name,operator_roles,\
             targets,groups,detail_key,detail_params"
        )
    );
    let row = lines.next().unwrap();
    assert!(row.contains(",GOVERNED_ACTION_RUN_BATCH_ACTION,AUDIT_OUTCOME_SUCCEEDED,main,"));
    assert!(
        row.contains(",'=cmd,'=cmd,"),
        "formulas are neutralised: {row}"
    );
    assert!(row.contains(",***BCDE,"), "targets stay masked: {row}");
    assert!(row.contains("test.done"));
    assert_eq!(lines.next(), None);

    let jsonl_path = directory.path().join("audit.jsonl");
    let result = export(
        &governance,
        AuditExportFormat::Jsonl,
        &jsonl_path,
        AuditQuery::default(),
    );
    let entries = std::fs::read_to_string(&jsonl_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<AuditEntry>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(result.exported as usize, entries.len());
    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries[0].action,
        GovernedAction::ExportAudit,
        "the earlier export is audited too"
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
fn refuses_exports_it_cannot_or_may_not_write() {
    let (directory, governance) = governance_with_store();
    let csv_path = directory.path().join("audit.csv");
    let refusal = |governance: &Arc<Governance>, format, path: &Path| {
        let result = export(governance, format, path, AuditQuery::default());
        assert!(!result.accepted);
        result.detail.unwrap().key
    };

    assert_eq!(
        refusal(&governance, AuditExportFormat::Unspecified, &csv_path),
        AUDIT_EXPORT_FORMAT_KEY
    );
    assert_eq!(
        refusal(&governance, AuditExportFormat::Csv, Path::new("audit.csv")),
        AUDIT_EXPORT_PATH_KEY
    );
    assert!(
        governance
            .query_audit(&AuditQuery::default())
            .entries
            .is_empty(),
        "malformed requests are refused before governance"
    );
    assert_eq!(
        refusal(
            &governance,
            AuditExportFormat::Csv,
            &directory.path().join("missing").join("audit.csv")
        ),
        AUDIT_EXPORT_PATH_KEY
    );

    let denied = Arc::new(Governance::assemble(GovernanceParts {
        authorization: Some(Box::new(DenyAll)),
        ..GovernanceParts::default()
    }));
    assert_eq!(
        refusal(&denied, AuditExportFormat::Csv, &csv_path),
        "test.denied"
    );
    assert!(!csv_path.exists());

    struct HideTrail;
    impl AuthorizationProvider for HideTrail {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn authorize(&self, context: &ActionContext<'_>) -> Decision {
            if context.action == GovernedAction::ViewAudit {
                Decision::Deny(ui_text("test.hidden"))
            } else {
                Decision::Allow
            }
        }
    }
    let hidden = Arc::new(Governance::assemble(GovernanceParts {
        authorization: Some(Box::new(HideTrail)),
        ..GovernanceParts::default()
    }));
    assert_eq!(
        refusal(&hidden, AuditExportFormat::Csv, &csv_path),
        "test.hidden",
        "a trail the operator may not see may not be exported either"
    );
    assert!(!csv_path.exists());
}
