use yunheshiguang_sdk::documents::DocumentId;

use crate::workspace::contract::{
    DeviceGroupDefinition, DeviceProfile, DeviceProfilePatch, UiText,
};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::book::Inventory;

pub const MAX_ALIAS_CHARS: usize = 32;
pub const MAX_NOTE_CHARS: usize = 200;
pub const MAX_TAG_CHARS: usize = 24;
pub const MAX_TAGS: usize = 12;
pub const MAX_NUMBER: u32 = 99_999;
pub const MAX_GROUP_NAME_CHARS: usize = 32;
pub const UNGROUPED_ID: &str = "ungrouped";
const RESERVED_GROUP_IDS: &[&str] = &["all", UNGROUPED_ID];

fn tidy(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn longer_than(value: &str, max: usize) -> bool {
    value.chars().count() > max
}

fn tags(values: &[String]) -> Result<Vec<String>, UiText> {
    let mut tags = Vec::<String>::new();

    for tag in values
        .iter()
        .map(|value| tidy(value))
        .filter(|tag| !tag.is_empty())
    {
        if longer_than(&tag, MAX_TAG_CHARS) {
            return Err(ui_text_with_params(
                ORGANIZE_TAG_TOO_LONG_KEY,
                "",
                [("max", MAX_TAG_CHARS.to_string())],
            ));
        }

        if !tags.iter().any(|known| known.eq_ignore_ascii_case(&tag)) {
            tags.push(tag);
        }
    }

    Ok(tags)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckedPatch {
    number: Option<u32>,
    alias: Option<String>,
    note: Option<String>,
    add_tags: Vec<String>,
    remove_tags: Vec<String>,
    group_id: Option<String>,
}

impl CheckedPatch {
    pub fn destination_group(&self) -> Option<String> {
        self.group_id.as_ref().map(|id| {
            if id.is_empty() {
                UNGROUPED_ID.to_string()
            } else {
                id.clone()
            }
        })
    }

    pub fn check(patch: &DeviceProfilePatch, device_count: usize) -> Result<Self, UiText> {
        if device_count == 0 {
            return Err(ui_text(ORGANIZE_NO_DEVICES_KEY));
        }

        if let Some(first) = patch.number.filter(|first| *first > 0)
            && u64::from(first) + device_count as u64 - 1 > u64::from(MAX_NUMBER)
        {
            return Err(ui_text_with_params(
                ORGANIZE_NUMBER_TOO_LARGE_KEY,
                "",
                [("max", MAX_NUMBER.to_string())],
            ));
        }

        let alias = patch.alias.as_deref().map(tidy);
        if alias
            .as_deref()
            .is_some_and(|alias| longer_than(alias, MAX_ALIAS_CHARS))
        {
            return Err(ui_text_with_params(
                ORGANIZE_ALIAS_TOO_LONG_KEY,
                "",
                [("max", MAX_ALIAS_CHARS.to_string())],
            ));
        }

        let note = patch.note.as_deref().map(|note| note.trim().to_string());
        if note
            .as_deref()
            .is_some_and(|note| longer_than(note, MAX_NOTE_CHARS))
        {
            return Err(ui_text_with_params(
                ORGANIZE_NOTE_TOO_LONG_KEY,
                "",
                [("max", MAX_NOTE_CHARS.to_string())],
            ));
        }

        Ok(Self {
            number: patch.number,
            alias,
            note,
            add_tags: tags(&patch.add_tags)?,
            remove_tags: tags(&patch.remove_tags)?,
            group_id: patch.group_id.as_deref().map(str::trim).map(str::to_string),
        })
    }

    pub fn check_group(&self, inventory: &Inventory) -> Result<(), UiText> {
        match self.group_id.as_deref() {
            Some(id) if !id.is_empty() && inventory.group(id).is_none() => {
                Err(ui_text(ORGANIZE_GROUP_MISSING_KEY))
            }
            _ => Ok(()),
        }
    }

    pub fn apply(&self, profile: &mut DeviceProfile, index: usize) -> Result<(), UiText> {
        if let Some(first) = self.number {
            profile.number = if first == 0 {
                0
            } else {
                first + u32::try_from(index).unwrap_or(u32::MAX)
            };
        }
        if let Some(alias) = &self.alias {
            profile.alias.clone_from(alias);
        }
        if let Some(note) = &self.note {
            profile.note.clone_from(note);
        }
        if let Some(group_id) = &self.group_id {
            profile.group_id.clone_from(group_id);
        }

        profile.tags.retain(|tag| {
            !self
                .remove_tags
                .iter()
                .any(|removed| removed.eq_ignore_ascii_case(tag))
        });
        for tag in &self.add_tags {
            if !profile
                .tags
                .iter()
                .any(|known| known.eq_ignore_ascii_case(tag))
            {
                profile.tags.push(tag.clone());
            }
        }

        if profile.tags.len() > MAX_TAGS {
            return Err(ui_text_with_params(
                ORGANIZE_TOO_MANY_TAGS_KEY,
                "",
                [("max", MAX_TAGS.to_string())],
            ));
        }

        Ok(())
    }
}

pub fn check_group(group: Option<DeviceGroupDefinition>) -> Result<DeviceGroupDefinition, UiText> {
    let group = group.unwrap_or_default();
    let id = group.id.trim().to_string();

    if DocumentId::parse(&id).is_err() || RESERVED_GROUP_IDS.contains(&id.as_str()) {
        return Err(ui_text(ORGANIZE_GROUP_ID_INVALID_KEY));
    }

    let name = tidy(&group.name);

    if name.is_empty() {
        return Err(ui_text(ORGANIZE_GROUP_NAME_REQUIRED_KEY));
    }

    if longer_than(&name, MAX_GROUP_NAME_CHARS) {
        return Err(ui_text_with_params(
            ORGANIZE_GROUP_NAME_TOO_LONG_KEY,
            "",
            [("max", MAX_GROUP_NAME_CHARS.to_string())],
        ));
    }

    Ok(DeviceGroupDefinition {
        id,
        name,
        tone: group.tone,
        ..Default::default()
    })
}
