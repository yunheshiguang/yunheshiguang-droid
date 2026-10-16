use std::collections::HashSet;

use crate::workspace::contract::{CloudPhone, ConnectionType};
use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};

use super::transport::adb_serial;

pub struct CloudSection {
    serials: HashSet<String>,
}

impl CloudSection {
    pub fn new(phones: &[CloudPhone]) -> Self {
        Self {
            serials: phones.iter().filter_map(adb_serial).collect(),
        }
    }
}

impl SnapshotSection for CloudSection {
    fn id(&self) -> &'static str {
        "cloud"
    }

    fn contribute(&self, draft: &mut SnapshotDraft) {
        draft.annotate_devices(|device| {
            if self.serials.contains(&device.id) {
                device.connection = ConnectionType::Cloud.into();
            }
        });
    }
}
