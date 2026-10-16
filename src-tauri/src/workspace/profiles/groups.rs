use std::collections::BTreeSet;
use std::sync::Arc;

use crate::workspace::adb::DeviceIdentities;
use crate::workspace::governance::GroupDirectory;

use super::book::ProfileBook;
use super::keys::device_keys;
use super::patch::UNGROUPED_ID;

pub struct ProfileGroups {
    pub book: Arc<ProfileBook>,
    pub identities: Arc<DeviceIdentities>,
}

impl GroupDirectory for ProfileGroups {
    fn groups_of(&self, targets: &[String]) -> Vec<String> {
        if targets.is_empty() {
            return Vec::new();
        }

        let known = self.identities.known();
        let serials = known
            .keys()
            .chain(targets.iter().filter(|target| !known.contains_key(*target)))
            .cloned()
            .collect::<Vec<_>>();
        let keys = device_keys(&serials, &known);
        let inventory = self.book.inventory();

        targets
            .iter()
            .map(|target| {
                let key = keys.get(target).unwrap_or(target);

                inventory
                    .profile(key, target)
                    .map(|profile| profile.group_id.as_str())
                    .filter(|id| inventory.group(id).is_some())
                    .unwrap_or(UNGROUPED_ID)
                    .to_string()
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
