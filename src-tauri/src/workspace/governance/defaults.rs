use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;
use yunheshiguang_sdk::governance::{
    ActionContext, AuditSink, AuthorizationProvider, Decision, EntitlementProvider,
    provider_info as provider,
};

use crate::workspace::contract::{AuditEntry, AuditOutcome, GovernanceProvider, GovernedAction};
use crate::workspace::messages::*;

const AUDIT_LOG_TARGET: &str = "yunheshiguang::audit";

pub struct LocalOperatorAuthorization;

impl AuthorizationProvider for LocalOperatorAuthorization {
    fn describe(&self) -> GovernanceProvider {
        provider(
            "core.local-operator",
            GOVERNANCE_LOCAL_OPERATOR_LABEL_KEY,
            GOVERNANCE_LOCAL_OPERATOR_DESCRIPTION_KEY,
        )
    }

    fn authorize(&self, _context: &ActionContext<'_>) -> Decision {
        Decision::Allow
    }
}

pub struct CoreEntitlements;

impl EntitlementProvider for CoreEntitlements {
    fn describe(&self) -> GovernanceProvider {
        provider(
            "core.open-source",
            GOVERNANCE_CORE_ENTITLEMENTS_LABEL_KEY,
            GOVERNANCE_CORE_ENTITLEMENTS_DESCRIPTION_KEY,
        )
    }

    fn granted_capabilities(&self) -> Vec<String> {
        Vec::new()
    }
}

pub struct LogAuditSink;

impl AuditSink for LogAuditSink {
    fn describe(&self) -> GovernanceProvider {
        provider(
            "core.run-log",
            GOVERNANCE_RUN_LOG_LABEL_KEY,
            GOVERNANCE_RUN_LOG_DESCRIPTION_KEY,
        )
    }

    fn record(&self, entry: &AuditEntry) {
        let action = GovernedAction::known(entry.action).unwrap_or_default();
        let outcome = AuditOutcome::known(entry.outcome).unwrap_or_default();

        log::info!(
            target: AUDIT_LOG_TARGET,
            "{} {} by {} on [{}]",
            action.proto_name(),
            outcome.proto_name(),
            entry.actor,
            entry.targets.join(", ")
        );
    }
}
