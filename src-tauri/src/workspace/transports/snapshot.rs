use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};

use super::registry::TransportRegistry;

pub struct TransportsSection<'a> {
    transports: &'a TransportRegistry,
}

impl<'a> TransportsSection<'a> {
    pub fn new(transports: &'a TransportRegistry) -> Self {
        Self { transports }
    }
}

impl SnapshotSection for TransportsSection<'_> {
    fn id(&self) -> &'static str {
        "transports"
    }

    fn contribute(&self, draft: &mut SnapshotDraft) {
        draft.add_connection_plans(self.transports.plans());
    }
}
