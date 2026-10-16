use std::collections::HashSet;
use yunheshiguang_contract::KnownEnum;

use crate::workspace::contract::{
    DeviceRecord, DeviceStatus, RunTarget, UiText, WorkspaceSnapshot, run_target::Target,
};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

pub fn check_target(target: Option<&RunTarget>) -> Result<&Target, UiText> {
    let target = target
        .and_then(|target| target.target.as_ref())
        .ok_or_else(|| ui_text(AUTOMATION_TARGET_MISSING_KEY))?;
    let filled = match target {
        Target::Devices(list) => list.device_ids.iter().any(|id| !id.trim().is_empty()),
        Target::GroupId(id) => !id.trim().is_empty(),
        Target::Tag(tag) => !tag.trim().is_empty(),
    };

    if filled {
        Ok(target)
    } else {
        Err(ui_text(AUTOMATION_TARGET_MISSING_KEY))
    }
}

fn online(device: &DeviceRecord) -> bool {
    matches!(
        DeviceStatus::known(device.status),
        Some(DeviceStatus::Online | DeviceStatus::Busy | DeviceStatus::Idle)
    )
}

fn reachable(device: &DeviceRecord, offline: bool) -> bool {
    online(device) || (offline && DeviceStatus::known(device.status) == Some(DeviceStatus::Offline))
}

fn tagged(device: &DeviceRecord, tag: &str) -> bool {
    let tag = tag.trim().to_lowercase();

    device.labels.as_option().is_some_and(|labels| {
        labels
            .tags
            .iter()
            .any(|candidate| candidate.to_lowercase() == tag)
    })
}

pub fn resolve_target(
    target: &Target,
    offline: bool,
    snapshot: impl FnOnce() -> WorkspaceSnapshot,
) -> Result<Vec<String>, UiText> {
    let devices = match target {
        Target::Devices(list) => {
            let mut seen = HashSet::new();

            list.device_ids
                .iter()
                .map(|id| id.trim())
                .filter(|id| !id.is_empty() && seen.insert(*id))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        }
        Target::GroupId(id) => snapshot()
            .devices
            .iter()
            .filter(|device| reachable(device, offline) && device.group_id == id.trim())
            .map(|device| device.id.clone())
            .collect(),
        Target::Tag(tag) => snapshot()
            .devices
            .iter()
            .filter(|device| reachable(device, offline) && tagged(device, tag))
            .map(|device| device.id.clone())
            .collect(),
    };

    if devices.is_empty() {
        Err(ui_text(AUTOMATION_NO_DEVICES_KEY))
    } else {
        Ok(devices)
    }
}
