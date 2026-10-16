use std::collections::HashMap;
use std::path::Path;

use yunheshiguang_sdk::vendor::DeviceIdentity;

use crate::workspace::adb::{AdbDevice, AdbDeviceState, DeviceIdentities};

const PLACEHOLDER_SERIALS: &[&str] = &["unknown", "0123456789abcdef", "0123456789", "emulator"];

fn hardware_serial(identity: &DeviceIdentity) -> Option<&str> {
    let serial = identity.serial_number.trim();

    (!serial.is_empty() && !PLACEHOLDER_SERIALS.contains(&serial.to_ascii_lowercase().as_str()))
        .then_some(serial)
}

pub fn device_keys(
    serials: &[String],
    identities: &HashMap<String, DeviceIdentity>,
) -> HashMap<String, String> {
    let hardware = |serial: &String| identities.get(serial).and_then(hardware_serial);
    let mut counts = HashMap::<&str, usize>::new();

    for serial in serials {
        if let Some(hardware) = hardware(serial) {
            *counts.entry(hardware).or_default() += 1;
        }
    }

    serials
        .iter()
        .map(|serial| {
            let key = hardware(serial)
                .filter(|hardware| counts.get(hardware) == Some(&1))
                .map_or_else(|| serial.clone(), str::to_string);

            (serial.clone(), key)
        })
        .collect()
}

pub fn connected_device_keys(
    devices: &[AdbDevice],
    adb_path: Option<&Path>,
    identities: &DeviceIdentities,
) -> HashMap<String, String> {
    let serials = devices
        .iter()
        .map(|device| device.serial.clone())
        .collect::<Vec<_>>();
    identities.retain_present(serials.iter().map(String::as_str));

    let resolved = adb_path
        .map(|adb_path| {
            let online = devices
                .iter()
                .filter(|device| device.state == AdbDeviceState::Online)
                .map(|device| device.serial.clone())
                .collect::<Vec<_>>();

            identities.resolve(adb_path, &online)
        })
        .unwrap_or_default();

    device_keys(&serials, &resolved)
}
