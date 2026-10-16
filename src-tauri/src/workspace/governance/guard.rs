use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use yunheshiguang_sdk::api::ApiIdentity;
use yunheshiguang_sdk::governance::{
    ActionContext, AuditSink, AuditStore, AuthorizationProvider, EntitlementProvider,
    IdentityProvider,
};

use super::defaults::{CoreEntitlements, LocalOperatorAuthorization, LogAuditSink};
use super::groups::GroupDirectory;
use super::identity::LocalUserIdentity;
use super::principal::{API_ACTOR_PREFIX, Principal};
use super::trail::{DEFAULT_AUDIT_CAPACITY, MemoryAuditTrail};
use crate::workspace::clock;
use crate::workspace::contract::{
    AuditEntry, AuditOutcome, AuditPage, AuditQuery, GovernanceState, GovernedAction, Operator,
    UiText,
};
use crate::workspace::messages::{
    AUDIT_UNAVAILABLE_KEY, GOVERNANCE_AUDIT_INTERRUPTED_KEY, GOVERNANCE_AUDIT_UNFINISHED_KEY,
};
use crate::workspace::text::{mask_device_id, ui_text};

#[derive(Default)]
pub struct GovernanceParts {
    pub authorization: Option<Box<dyn AuthorizationProvider>>,
    pub entitlements: Option<Box<dyn EntitlementProvider>>,
    pub identity: Option<Box<dyn IdentityProvider>>,
    pub audit_sinks: Vec<Box<dyn AuditSink>>,
    pub audit_store: Option<Arc<dyn AuditStore>>,
}

struct Pending {
    action: GovernedAction,
    principal: Principal,
    targets: Vec<String>,
    groups: Vec<String>,
}

pub struct Governance {
    authorization: Box<dyn AuthorizationProvider>,
    entitlements: Box<dyn EntitlementProvider>,
    identity: Box<dyn IdentityProvider>,
    trail: MemoryAuditTrail,
    sinks: Vec<Box<dyn AuditSink>>,
    store: OnceLock<Arc<dyn AuditStore>>,
    groups: OnceLock<Arc<dyn GroupDirectory>>,
    pending: Mutex<HashMap<u64, Pending>>,
    next_ticket: AtomicU64,
}

impl Governance {
    #[cfg(test)]
    pub fn new(
        authorization: Box<dyn AuthorizationProvider>,
        entitlements: Box<dyn EntitlementProvider>,
    ) -> Self {
        Self::assemble(GovernanceParts {
            authorization: Some(authorization),
            entitlements: Some(entitlements),
            ..GovernanceParts::default()
        })
    }

    #[cfg(test)]
    pub fn core() -> Self {
        Self::assemble(GovernanceParts::default())
    }

    pub fn assemble(parts: GovernanceParts) -> Self {
        let governance = Self {
            authorization: parts
                .authorization
                .unwrap_or_else(|| Box::new(LocalOperatorAuthorization)),
            entitlements: parts
                .entitlements
                .unwrap_or_else(|| Box::new(CoreEntitlements)),
            identity: parts
                .identity
                .unwrap_or_else(|| Box::new(LocalUserIdentity::detect())),
            trail: MemoryAuditTrail::new(DEFAULT_AUDIT_CAPACITY),
            sinks: std::iter::once(Box::new(LogAuditSink) as Box<dyn AuditSink>)
                .chain(parts.audit_sinks)
                .collect(),
            store: OnceLock::new(),
            groups: OnceLock::new(),
            pending: Mutex::new(HashMap::new()),
            next_ticket: AtomicU64::new(0),
        };

        if let Some(store) = parts.audit_store {
            governance.install_audit_store(store);
        }

        governance
    }

    #[cfg(test)]
    pub fn with_audit_sink(mut self, sink: Box<dyn AuditSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    pub fn install_audit_store(&self, store: Arc<dyn AuditStore>) -> bool {
        self.store.set(store).is_ok()
    }

    pub fn has_audit_store(&self) -> bool {
        self.store.get().is_some()
    }

    pub fn flush_audit(&self) {
        let interrupted = self
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain()
            .map(|(_, pending)| pending)
            .collect::<Vec<_>>();

        for pending in interrupted {
            self.write(
                pending,
                AuditOutcome::Failed,
                Some(ui_text(GOVERNANCE_AUDIT_INTERRUPTED_KEY)),
            );
        }
        if let Some(store) = self.store.get() {
            store.flush();
        }
    }

    pub fn install_group_directory(&self, directory: Arc<dyn GroupDirectory>) {
        let _ = self.groups.set(directory);
    }

    pub fn current_operator(&self) -> Operator {
        self.identity.current()
    }

    pub fn principal(&self, actor: impl Into<String>) -> Principal {
        Principal::new(actor, self.current_operator())
    }

    pub fn api_principal(&self, caller: &ApiIdentity) -> Principal {
        let operator = caller
            .vouched
            .then(|| self.identity.resolve(&caller.name))
            .flatten()
            .unwrap_or_else(|| Operator {
                id: format!("{API_ACTOR_PREFIX}{}", caller.name),
                name: caller.name.clone(),
                roles: caller.roles.clone(),
                ..Default::default()
            });

        Principal::new(format!("{API_ACTOR_PREFIX}{}", caller.name), operator)
    }

    fn affected_groups(&self, targets: &[String], also: &[String]) -> Vec<String> {
        let mut groups = self
            .groups
            .get()
            .map(|directory| directory.groups_of(targets))
            .unwrap_or_default();
        groups.extend(also.iter().filter(|group| !group.is_empty()).cloned());
        groups.sort();
        groups.dedup();
        groups
    }

    fn decide(
        &self,
        action: GovernedAction,
        principal: &Principal,
        targets: &[String],
        groups: &[String],
    ) -> Result<(), UiText> {
        let context = ActionContext::new(action, &principal.actor, targets)
            .with_operator(&principal.operator)
            .with_groups(groups);

        self.authorization.authorize(&context).into_result()
    }

    fn deny(
        &self,
        action: GovernedAction,
        principal: &Principal,
        targets: &[String],
        groups: Vec<String>,
        reason: &UiText,
    ) {
        self.write(
            Pending {
                action,
                principal: principal.clone(),
                targets: masked(targets),
                groups,
            },
            AuditOutcome::Denied,
            Some(reason.clone()),
        );
    }

    pub fn begin(
        self: &Arc<Self>,
        action: GovernedAction,
        principal: &Principal,
        targets: &[String],
    ) -> Result<AuditTicket, UiText> {
        self.begin_in(action, principal, targets, &[])
    }

    pub fn begin_in(
        self: &Arc<Self>,
        action: GovernedAction,
        principal: &Principal,
        targets: &[String],
        also_groups: &[String],
    ) -> Result<AuditTicket, UiText> {
        let groups = self.affected_groups(targets, also_groups);

        match self.decide(action, principal, targets, &groups) {
            Ok(()) => {
                let id = self.next_ticket.fetch_add(1, Ordering::Relaxed);
                self.pending
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(
                        id,
                        Pending {
                            action,
                            principal: principal.clone(),
                            targets: masked(targets),
                            groups,
                        },
                    );
                Ok(AuditTicket {
                    governance: Arc::clone(self),
                    id,
                    finished: false,
                })
            }
            Err(reason) => {
                self.deny(action, principal, targets, groups, &reason);
                Err(reason)
            }
        }
    }

    pub fn authorize(
        &self,
        action: GovernedAction,
        principal: &Principal,
        targets: &[String],
    ) -> Result<(), UiText> {
        let groups = self.affected_groups(targets, &[]);

        self.decide(action, principal, targets, &groups)
            .inspect_err(|reason| self.deny(action, principal, targets, groups, reason))
    }

    pub fn permits(&self, action: GovernedAction, principal: &Principal) -> bool {
        self.decide(action, principal, &[], &[]).is_ok()
    }

    pub fn audit_interrupted(
        &self,
        action: GovernedAction,
        principal: &Principal,
        targets: &[String],
    ) {
        let groups = self.affected_groups(targets, &[]);

        self.write(
            Pending {
                action,
                principal: principal.clone(),
                targets: masked(targets),
                groups,
            },
            AuditOutcome::Failed,
            Some(ui_text(GOVERNANCE_AUDIT_INTERRUPTED_KEY)),
        );
    }

    pub fn state(&self) -> GovernanceState {
        GovernanceState {
            authorization: self.authorization.describe().into(),
            entitlements: self.entitlements.describe().into(),
            audit_sinks: std::iter::once(self.trail.describe())
                .chain(self.sinks.iter().map(|sink| sink.describe()))
                .collect(),
            granted_capabilities: self.granted_capabilities(),
            audit_trail: self.trail.recent(),
            audit_capacity: u32::try_from(self.trail.capacity()).unwrap_or(u32::MAX),
            license: self.entitlements.license().into(),
            identity: self.identity.describe().into(),
            operator: self.current_operator().into(),
            audit_store: (self.store.get().map(|store| store.describe())).into(),
            ..Default::default()
        }
    }

    pub fn state_for(&self, principal: &Principal) -> GovernanceState {
        let mut state = self.state();
        if !self.permits(GovernedAction::ViewAudit, principal) {
            state.audit_trail.clear();
        }
        state
    }

    pub fn query_audit(&self, query: &AuditQuery) -> AuditPage {
        let unavailable = |problem: UiText| AuditPage {
            problem: problem.into(),
            ..AuditPage::default()
        };

        match self.store.get() {
            Some(store) => store.query(query).unwrap_or_else(unavailable),
            None => unavailable(ui_text(AUDIT_UNAVAILABLE_KEY)),
        }
    }

    pub fn granted_capabilities(&self) -> Vec<String> {
        self.entitlements.granted_capabilities()
    }

    fn settle(&self, id: u64, outcome: AuditOutcome, detail: Option<UiText>) {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&id);

        if let Some(pending) = pending {
            self.write(pending, outcome, detail);
        }
    }

    fn write(&self, pending: Pending, outcome: AuditOutcome, detail: Option<UiText>) {
        let entry = AuditEntry {
            id: uuid::Uuid::now_v7().to_string(),
            occurred_at: clock::now().into(),
            action: pending.action.into(),
            outcome: outcome.into(),
            actor: pending.principal.actor,
            targets: pending.targets,
            detail: detail.into(),
            operator: pending.principal.operator.into(),
            groups: pending.groups,
            ..Default::default()
        };

        self.trail.record(&entry);

        if let Some(store) = self.store.get() {
            store.append(&entry);
        }

        for sink in &self.sinks {
            let recorded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                sink.record(&entry);
            }));
            if recorded.is_err() {
                log::error!("governance: an audit sink failed while recording an entry");
            }
        }
    }
}

fn masked(targets: &[String]) -> Vec<String> {
    targets
        .iter()
        .map(|target| mask_device_id(target))
        .collect()
}

#[must_use = "an unfinished ticket is audited as a failure"]
pub struct AuditTicket {
    governance: Arc<Governance>,
    id: u64,
    finished: bool,
}

impl AuditTicket {
    pub fn finish(mut self, outcome: AuditOutcome, detail: Option<UiText>) {
        self.finished = true;
        self.governance.settle(self.id, outcome, detail);
    }
}

impl Drop for AuditTicket {
    fn drop(&mut self) {
        if !self.finished {
            self.governance.settle(
                self.id,
                AuditOutcome::Failed,
                Some(ui_text(GOVERNANCE_AUDIT_UNFINISHED_KEY)),
            );
        }
    }
}
