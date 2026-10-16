use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};

use yunheshiguang_sdk::governance::{AuditSink, provider_info as provider};

use crate::workspace::contract::{AuditEntry, GovernanceProvider};
use crate::workspace::messages::*;

pub const DEFAULT_AUDIT_CAPACITY: usize = 200;

pub struct MemoryAuditTrail {
    capacity: usize,
    entries: Mutex<VecDeque<AuditEntry>>,
}

impl MemoryAuditTrail {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn recent(&self) -> Vec<AuditEntry> {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }
}

impl AuditSink for MemoryAuditTrail {
    fn describe(&self) -> GovernanceProvider {
        provider(
            "core.session-trail",
            GOVERNANCE_SESSION_TRAIL_LABEL_KEY,
            GOVERNANCE_SESSION_TRAIL_DESCRIPTION_KEY,
        )
    }

    fn record(&self, entry: &AuditEntry) {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);

        entries.push_front(entry.clone());
        entries.truncate(self.capacity);
    }
}
