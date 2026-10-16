use std::collections::HashMap;
use yunheshiguang_contract::KnownEnum;

use crate::workspace::contract::{
    DeviceGroup, DeviceLabels, DeviceProfile, DeviceRecord, DeviceStatus, GroupTone,
};
use crate::workspace::messages::DEVICE_GROUP_DEFAULT_KEY;
use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};
use crate::workspace::text::{literal_text, note_text, ui_text};

use super::book::Inventory;
use super::patch::UNGROUPED_ID;

pub struct ProfilesSection<'a> {
    inventory: &'a Inventory,
    keys: &'a HashMap<String, String>,
}

impl<'a> ProfilesSection<'a> {
    pub fn new(inventory: &'a Inventory, keys: &'a HashMap<String, String>) -> Self {
        Self { inventory, keys }
    }

    fn profile_of(&self, device_id: &str) -> Option<&DeviceProfile> {
        let key = self.keys.get(device_id).map_or(device_id, String::as_str);

        self.inventory.profile(key, device_id)
    }

    fn label(&self, device: &mut DeviceRecord) {
        let profile = self.profile_of(&device.id);
        let group = profile
            .map(|profile| profile.group_id.as_str())
            .filter(|id| !id.is_empty())
            .and_then(|id| self.inventory.group(id));

        match group {
            Some(group) => {
                device.group_id.clone_from(&group.id);
                device.group = literal_text(group.name.clone()).into();
            }
            None => {
                device.group_id = UNGROUPED_ID.to_string();
                device.group = ui_text(DEVICE_GROUP_DEFAULT_KEY).into();
            }
        }

        device.labels = profile.map(labels).into();
    }

    fn summary(&self, devices: &[DeviceRecord]) -> Vec<DeviceGroup> {
        let mut groups = self
            .inventory
            .groups
            .iter()
            .map(|group| DeviceGroup {
                id: group.id.clone(),
                name: literal_text(group.name.clone()).into(),
                total: 0,
                online: 0,
                tone: if group.tone == GroupTone::Unspecified {
                    GroupTone::Emerald.into()
                } else {
                    group.tone
                },
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let mut ungrouped = DeviceGroup {
            id: UNGROUPED_ID.to_string(),
            name: ui_text(DEVICE_GROUP_DEFAULT_KEY).into(),
            total: 0,
            online: 0,
            tone: GroupTone::Slate.into(),
            ..Default::default()
        };

        for device in devices {
            let entry = groups
                .iter_mut()
                .find(|group| group.id == device.group_id)
                .unwrap_or(&mut ungrouped);

            entry.total += 1;
            if matches!(
                DeviceStatus::known(device.status),
                Some(DeviceStatus::Online | DeviceStatus::Busy | DeviceStatus::Idle)
            ) {
                entry.online += 1;
            }
        }

        if ungrouped.total > 0 {
            groups.push(ungrouped);
        }

        groups
    }
}

fn labels(profile: &DeviceProfile) -> DeviceLabels {
    DeviceLabels {
        number: profile.number,
        alias: profile.alias.clone(),
        note: ((!profile.note.is_empty()).then(|| note_text(profile.note.clone()))).into(),
        tags: profile.tags.clone(),
        ..Default::default()
    }
}

impl SnapshotSection for ProfilesSection<'_> {
    fn id(&self) -> &'static str {
        "profiles"
    }

    fn contribute(&self, draft: &mut SnapshotDraft) {
        draft.annotate_devices(|device| self.label(device));
        let groups = self.summary(draft.devices());
        draft.add_groups(groups);
    }
}
