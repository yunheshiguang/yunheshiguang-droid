use std::sync::{Arc, Mutex};

use yunheshiguang_sdk::api::ApiIdentity;
use yunheshiguang_sdk::governance::{
    ActionContext, AuditSink, AuthorizationProvider, Decision, EntitlementProvider,
    IdentityProvider, provider_info as provider,
};

use super::identity::{LocalUserIdentity, OWNER_ROLE};
use super::trail::MemoryAuditTrail;
use super::{Governance, GovernanceParts, GroupDirectory};
use yunheshiguang_contract::yunheshiguang::governance::v1::{License, LicenseStatus};

use crate::workspace::contract::{
    AuditEntry, AuditOutcome, GovernanceProvider, GovernedAction, Operator,
};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

const DENIED_KEY: &str = "test.denied";

#[derive(Default)]
struct DenyAll {
    seen_targets: Arc<Mutex<Vec<String>>>,
}

impl AuthorizationProvider for DenyAll {
    fn describe(&self) -> GovernanceProvider {
        provider("test.deny-all", DENIED_KEY, DENIED_KEY)
    }

    fn authorize(&self, context: &ActionContext<'_>) -> Decision {
        self.seen_targets
            .lock()
            .unwrap()
            .extend_from_slice(context.targets);
        Decision::Deny(ui_text(DENIED_KEY))
    }
}

struct Grants(Vec<&'static str>);

impl EntitlementProvider for Grants {
    fn describe(&self) -> GovernanceProvider {
        provider("test.grants", DENIED_KEY, DENIED_KEY)
    }

    fn granted_capabilities(&self) -> Vec<String> {
        self.0.iter().map(|key| key.to_string()).collect()
    }
}

#[derive(Clone, Default)]
struct Collector(Arc<Mutex<Vec<AuditEntry>>>);

impl AuditSink for Collector {
    fn describe(&self) -> GovernanceProvider {
        provider("test.collector", DENIED_KEY, DENIED_KEY)
    }

    fn record(&self, entry: &AuditEntry) {
        self.0.lock().unwrap().push(entry.clone());
    }
}

fn targets() -> Vec<String> {
    vec!["192.168.1.20:5555".to_string(), "R58M20ABCDE".to_string()]
}

#[test]
fn audits_an_allowed_action_with_masked_targets() {
    let governance = Arc::new(Governance::core());

    governance
        .begin(
            GovernedAction::ConnectDevice,
            &governance.principal("main"),
            &targets(),
        )
        .expect("the local operator may do everything")
        .finish(AuditOutcome::Succeeded, None);

    let trail = governance.state().audit_trail;
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].action, GovernedAction::ConnectDevice);
    assert_eq!(trail[0].outcome, AuditOutcome::Succeeded);
    assert_eq!(trail[0].actor, "main");
    assert_eq!(trail[0].targets, vec!["***5555", "***BCDE"]);
    assert!(
        trail[0]
            .occurred_at
            .as_option()
            .is_some_and(|at| at.seconds > 0)
    );
}

#[test]
fn records_a_denial_and_hands_the_reason_back() {
    let deny = DenyAll::default();
    let seen = Arc::clone(&deny.seen_targets);
    let governance = Arc::new(Governance::new(
        Box::new(deny),
        Box::new(Grants(Vec::new())),
    ));

    let Err(reason) = governance.begin(
        GovernedAction::RunBatchAction,
        &governance.principal("main"),
        &targets(),
    ) else {
        panic!("the provider denies everything");
    };

    assert_eq!(reason.key, DENIED_KEY);
    assert_eq!(*seen.lock().unwrap(), targets(), "policy sees raw targets");

    let trail = governance.state().audit_trail;
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].outcome, AuditOutcome::Denied);
    assert_eq!(trail[0].detail.as_option().unwrap().key, DENIED_KEY);
    assert_eq!(trail[0].targets, vec!["***5555", "***BCDE"]);
}

#[test]
fn audits_a_ticket_dropped_without_a_result_as_a_failure() {
    let governance = Arc::new(Governance::core());

    drop(
        governance
            .begin(
                GovernedAction::DisconnectDevice,
                &governance.principal("main"),
                &targets(),
            )
            .unwrap(),
    );

    let trail = governance.state().audit_trail;
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].outcome, AuditOutcome::Failed);
    assert_eq!(
        trail[0].detail.as_option().unwrap().key,
        GOVERNANCE_AUDIT_UNFINISHED_KEY
    );
}

struct Failing;

impl AuditSink for Failing {
    fn describe(&self) -> GovernanceProvider {
        provider("test.failing", DENIED_KEY, DENIED_KEY)
    }

    fn record(&self, _entry: &AuditEntry) {
        panic!("this sink cannot record");
    }
}

#[test]
fn keeps_auditing_when_an_added_sink_fails() {
    let collector = Collector::default();
    let governance = Arc::new(
        Governance::core()
            .with_audit_sink(Box::new(Failing))
            .with_audit_sink(Box::new(collector.clone())),
    );

    governance
        .begin(
            GovernedAction::PairDevice,
            &governance.principal("main"),
            &[],
        )
        .unwrap()
        .finish(AuditOutcome::Succeeded, None);

    assert_eq!(governance.state().audit_trail.len(), 1);
    assert_eq!(collector.0.lock().unwrap().len(), 1);
}

#[test]
fn forwards_every_entry_to_added_sinks_and_lists_them() {
    let collector = Collector::default();
    let governance = Arc::new(Governance::core().with_audit_sink(Box::new(collector.clone())));

    for action in [GovernedAction::PairDevice, GovernedAction::ConnectDevice] {
        governance
            .begin(action, &governance.principal("main"), &[])
            .unwrap()
            .finish(AuditOutcome::Failed, None);
    }

    assert_eq!(collector.0.lock().unwrap().len(), 2);

    let state = governance.state();
    let sink_ids: Vec<_> = state
        .audit_sinks
        .iter()
        .map(|sink| sink.id.as_str())
        .collect();
    assert_eq!(
        sink_ids,
        ["core.session-trail", "core.run-log", "test.collector"]
    );
    assert_eq!(
        state.audit_trail[0].action,
        GovernedAction::ConnectDevice,
        "newest entry first"
    );
    assert_ne!(state.audit_trail[0].id, state.audit_trail[1].id);
}

#[test]
fn reports_the_installed_providers_and_granted_capabilities() {
    let core = Governance::core().state();
    assert_eq!(core.authorization.unwrap().id, "core.local-operator");
    assert_eq!(core.entitlements.unwrap().id, "core.open-source");
    assert!(core.granted_capabilities.is_empty());
    assert!(core.audit_capacity > 0);

    let granted = Governance::new(
        Box::new(DenyAll::default()),
        Box::new(Grants(vec!["pro.audit"])),
    )
    .state();
    assert_eq!(granted.granted_capabilities, vec!["pro.audit"]);
}

struct Licensed;

impl EntitlementProvider for Licensed {
    fn describe(&self) -> GovernanceProvider {
        provider("test.licensed", DENIED_KEY, DENIED_KEY)
    }

    fn granted_capabilities(&self) -> Vec<String> {
        vec!["pro.market".to_string()]
    }

    fn license(&self) -> Option<License> {
        Some(License {
            plan_id: "team".to_string(),
            status: LicenseStatus::Active.into(),
            ..License::default()
        })
    }
}

#[test]
fn assembles_contributed_providers_around_the_built_in_sinks() {
    let collector = Collector::default();
    let governance = Governance::assemble(GovernanceParts {
        authorization: Some(Box::new(DenyAll::default())),
        entitlements: Some(Box::new(Licensed)),
        audit_sinks: vec![Box::new(collector)],
        ..GovernanceParts::default()
    })
    .state();

    assert_eq!(governance.authorization.unwrap().id, "test.deny-all");
    assert_eq!(governance.granted_capabilities, vec!["pro.market"]);
    assert_eq!(governance.license.unwrap().plan_id, "team");
    let sinks: Vec<_> = governance
        .audit_sinks
        .iter()
        .map(|sink| sink.id.as_str())
        .collect();
    assert_eq!(
        sinks,
        ["core.session-trail", "core.run-log", "test.collector"]
    );
}

#[test]
fn has_no_license_in_the_open_source_core() {
    assert!(Governance::core().state().license.is_unset());
}

struct ShelfA;

impl GroupDirectory for ShelfA {
    fn groups_of(&self, targets: &[String]) -> Vec<String> {
        if targets.is_empty() {
            Vec::new()
        } else {
            vec!["shelf-a".to_string()]
        }
    }
}

type Seen = Arc<Mutex<Vec<(Option<Operator>, Vec<String>)>>>;

#[derive(Default)]
struct Watcher {
    seen: Seen,
}

impl AuthorizationProvider for Watcher {
    fn describe(&self) -> GovernanceProvider {
        provider("test.watcher", DENIED_KEY, DENIED_KEY)
    }

    fn authorize(&self, context: &ActionContext<'_>) -> Decision {
        self.seen
            .lock()
            .unwrap()
            .push((context.operator.cloned(), context.groups.to_vec()));
        Decision::Allow
    }
}

fn alice() -> Option<Box<dyn IdentityProvider>> {
    Some(Box::new(LocalUserIdentity::new("alice", "Alice")))
}

#[test]
fn hands_the_operator_and_target_groups_to_authorization_and_audit() {
    let watcher = Watcher::default();
    let seen = Arc::clone(&watcher.seen);
    let governance = Arc::new(Governance::assemble(GovernanceParts {
        authorization: Some(Box::new(watcher)),
        identity: alice(),
        ..GovernanceParts::default()
    }));
    governance.install_group_directory(Arc::new(ShelfA));
    let principal = governance.principal("main");

    assert_eq!(
        principal.operator,
        Operator {
            id: "local:alice".to_string(),
            name: "Alice".to_string(),
            roles: vec![OWNER_ROLE.to_string()],
            ..Default::default()
        }
    );
    governance
        .begin(GovernedAction::RunBatchAction, &principal, &targets())
        .unwrap()
        .finish(AuditOutcome::Succeeded, None);

    let expected = (
        Some(principal.operator.clone()),
        vec!["shelf-a".to_string()],
    );
    assert_eq!(*seen.lock().unwrap(), [expected]);
    let state = governance.state();
    assert_eq!(
        state.audit_trail[0].operator,
        principal.operator.clone().into()
    );
    assert_eq!(state.audit_trail[0].groups, ["shelf-a"]);
    assert_eq!(state.identity.unwrap().id, "core.local-user");
    assert_eq!(state.operator, principal.operator.into());
}

#[test]
fn names_api_callers_by_the_identity_provider_or_by_their_roles() {
    let governance = Governance::assemble(GovernanceParts {
        identity: alice(),
        ..GovernanceParts::default()
    });

    let known = governance.api_principal(&ApiIdentity::new("local:alice").vouched());
    assert_eq!(known.actor, "api:local:alice");
    assert_eq!(known.operator.name, "Alice");

    let claimed = governance.api_principal(&ApiIdentity::new("local:alice"));
    assert_eq!(claimed.operator.name, "local:alice");
    assert!(
        claimed.operator.roles.is_empty(),
        "a token name is not an identity"
    );

    let bot = governance.api_principal(&ApiIdentity::new("ops-bot").with_roles(["operator"]));
    assert_eq!(bot.actor, "api:ops-bot");
    assert_eq!(
        bot.operator,
        Operator {
            id: "api:ops-bot".to_string(),
            name: "ops-bot".to_string(),
            roles: vec!["operator".to_string()],
            ..Default::default()
        }
    );
}

#[test]
fn falls_back_to_a_plain_local_operator_without_a_user_name() {
    let nameless = LocalUserIdentity::new("", "").current();
    assert_eq!(
        (nameless.id.as_str(), nameless.name.as_str()),
        ("local", "local")
    );

    let detected = LocalUserIdentity::detect().current();
    assert!(detected.id.starts_with("local"));
    assert_eq!(detected.roles, [OWNER_ROLE]);
}

#[test]
fn keeps_only_the_most_recent_entries() {
    let trail = MemoryAuditTrail::new(2);

    for id in ["a", "b", "c"] {
        trail.record(&AuditEntry {
            id: id.to_string(),
            ..AuditEntry::default()
        });
    }

    let ids: Vec<_> = trail.recent().into_iter().map(|entry| entry.id).collect();
    assert_eq!(ids, ["c", "b"]);
}

#[test]
fn records_actions_still_running_when_the_app_quits_as_interrupted() {
    let governance = Arc::new(Governance::core());
    let ticket = governance
        .begin(
            GovernedAction::RunAutomation,
            &governance.principal("main"),
            &targets(),
        )
        .unwrap();

    governance.flush_audit();
    ticket.finish(AuditOutcome::Succeeded, None);

    let trail = governance.state().audit_trail;
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].outcome, AuditOutcome::Failed);
    assert_eq!(
        trail[0].detail.as_option().unwrap().key,
        GOVERNANCE_AUDIT_INTERRUPTED_KEY
    );
}

#[test]
fn shows_authorization_the_groups_an_action_changes() {
    let watcher = Watcher::default();
    let seen = Arc::clone(&watcher.seen);
    let governance = Arc::new(Governance::assemble(GovernanceParts {
        authorization: Some(Box::new(watcher)),
        ..GovernanceParts::default()
    }));
    governance.install_group_directory(Arc::new(ShelfA));

    governance
        .begin_in(
            GovernedAction::OrganizeDevices,
            &governance.principal("main"),
            &targets(),
            &["shelf-b".to_string(), String::new()],
        )
        .unwrap()
        .finish(AuditOutcome::Succeeded, None);

    assert_eq!(seen.lock().unwrap()[0].1, ["shelf-a", "shelf-b"]);
    assert_eq!(
        governance.state().audit_trail[0].groups,
        ["shelf-a", "shelf-b"]
    );
}

#[test]
fn records_refused_reads_and_hides_the_trail_from_whoever_may_not_read_it() {
    let governance = Governance::new(Box::new(DenyAll::default()), Box::new(Grants(Vec::new())));
    let principal = governance.principal("api:bot");

    assert_eq!(
        governance
            .authorize(GovernedAction::ViewAudit, &principal, &[])
            .unwrap_err()
            .key,
        DENIED_KEY
    );
    assert_eq!(governance.state().audit_trail.len(), 1);
    assert!(governance.state_for(&principal).audit_trail.is_empty());
    assert!(!governance.permits(GovernedAction::ViewAudit, &principal));
    assert_eq!(governance.state().audit_trail.len(), 1);
}
