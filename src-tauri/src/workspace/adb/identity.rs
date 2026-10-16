use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use yunheshiguang_sdk::vendor::DeviceIdentity;

use super::{AdbClient, PROBE_POLICY, run_bounded};

const IDENTITY_CONCURRENCY: usize = 8;
const MAX_DEVICES_PER_REQUEST: usize = 500;

fn parse_getprop(stdout: &str) -> DeviceIdentity {
    DeviceIdentity::from_properties(stdout.lines().filter_map(|line| {
        let (key, value) = line.trim().split_once("]: [")?;

        Some((key.strip_prefix('[')?, value.strip_suffix(']')?))
    }))
}

fn read_identity(client: &AdbClient, serial: &str) -> Option<DeviceIdentity> {
    let output = client
        .run_serial_with_policy(serial, ["shell", "getprop"], PROBE_POLICY)
        .ok()?;
    let identity = parse_getprop(&String::from_utf8_lossy(&output.stdout));

    (identity != DeviceIdentity::default()).then_some(identity)
}

#[derive(Default)]
pub struct DeviceIdentities {
    cache: Mutex<HashMap<String, DeviceIdentity>>,
}

impl DeviceIdentities {
    pub fn resolve(&self, adb_path: &Path, serials: &[String]) -> HashMap<String, DeviceIdentity> {
        let mut known = HashMap::new();
        let mut unknown = Vec::new();

        {
            let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            let mut seen = HashSet::new();

            for serial in serials
                .iter()
                .filter(|serial| seen.insert(*serial))
                .take(MAX_DEVICES_PER_REQUEST)
            {
                match cache.get(serial) {
                    Some(identity) => {
                        known.insert(serial.clone(), identity.clone());
                    }
                    None => unknown.push(serial.clone()),
                }
            }
        }

        let fetched = run_bounded(
            unknown,
            IDENTITY_CONCURRENCY,
            || AdbClient::new(adb_path),
            |client, serial| read_identity(client, &serial).map(|identity| (serial, identity)),
        );
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);

        for (serial, identity) in fetched.into_iter().flatten() {
            cache.insert(serial.clone(), identity.clone());
            known.insert(serial, identity);
        }

        known
    }

    pub fn known(&self) -> HashMap<String, DeviceIdentity> {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn retain_present<'a>(&self, present: impl IntoIterator<Item = &'a str>) {
        let present = present.into_iter().collect::<HashSet<_>>();

        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|serial, _| present.contains(serial.as_str()));
    }

    #[cfg(test)]
    pub fn remember(&self, serial: &str, identity: DeviceIdentity) {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(serial.to_string(), identity);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const NO_ADB: &str = "/nonexistent/adb";

    #[test]
    fn reads_the_identity_properties_out_of_getprop() {
        let identity = parse_getprop(
            "[ro.build.version.sdk]: [34]\n\
[ro.product.brand]: [Redmi]\n\
[ro.product.manufacturer]: [Xiaomi]\n\
[ro.product.model]: [23078RKD5C]\n\
[ro.board.platform]: [taro]\n\
[ro.serialno]: [a1b2c3d4]\n\
garbage line\n",
        );

        assert_eq!(identity.brand, "Redmi");
        assert_eq!(identity.manufacturer, "Xiaomi");
        assert_eq!(identity.model, "23078RKD5C");
        assert_eq!(identity.platform, "taro");
        assert_eq!(identity.serial_number, "a1b2c3d4");
        assert_eq!(identity.sdk_version, "34");
    }

    #[test]
    fn forgets_devices_that_are_gone() {
        let identities = DeviceIdentities::default();
        let redmi = DeviceIdentity::from_properties([("ro.product.brand", "Redmi")]);

        identities.remember("usb-1", redmi.clone());
        identities.remember("10.0.0.8:5555", redmi);
        identities.retain_present(["usb-1"]);

        let known = identities.resolve(Path::new(NO_ADB), &["usb-1".to_string()]);
        assert_eq!(known.len(), 1);
        assert!(
            identities
                .cache
                .lock()
                .unwrap()
                .get("10.0.0.8:5555")
                .is_none()
        );
    }
}
