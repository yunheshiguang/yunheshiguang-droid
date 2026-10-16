use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use yunheshiguang_sdk::vendor::{DeviceFailure, DeviceIdentity, VendorAdapter};

use crate::workspace::adb::DeviceIdentities;
use crate::workspace::contract::{
    AdbDeviceActionResult, BatchActionKind, DeviceHint, DeviceVendor, DeviceVendors,
};

use super::builtin::builtin_adapters;

pub struct VendorRegistry {
    adapters: Vec<Box<dyn VendorAdapter>>,
    identities: Arc<DeviceIdentities>,
}

impl VendorRegistry {
    pub fn new(
        contributed: Vec<Box<dyn VendorAdapter>>,
        identities: Arc<DeviceIdentities>,
    ) -> Self {
        Self {
            adapters: contributed.into_iter().chain(builtin_adapters()).collect(),
            identities,
        }
    }

    fn recognize(&self, identity: &DeviceIdentity) -> Option<&dyn VendorAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.recognizes(identity))
            .map(AsRef::as_ref)
    }

    pub fn vendors(&self, adb_path: &Path, serials: &[String]) -> DeviceVendors {
        let identities = self.identities.resolve(adb_path, serials);
        let mut vendors = DeviceVendors::default();
        let mut profiled = HashSet::new();

        for serial in serials {
            let Some(adapter) = identities
                .get(serial)
                .and_then(|identity| self.recognize(identity))
            else {
                continue;
            };
            let profile = adapter.profile();

            vendors.devices.push(DeviceVendor {
                device_id: serial.clone(),
                vendor_id: profile.id.clone(),
                ..Default::default()
            });
            if profiled.insert(profile.id.clone()) {
                vendors.vendors.push(profile);
            }
        }

        vendors
    }

    pub fn hints(
        &self,
        adb_path: &Path,
        action: BatchActionKind,
        results: &[AdbDeviceActionResult],
    ) -> Vec<DeviceHint> {
        let failed = results
            .iter()
            .filter(|result| !result.success)
            .collect::<Vec<_>>();

        if failed.is_empty() {
            return Vec::new();
        }

        let serials = failed
            .iter()
            .map(|result| result.device_id.clone())
            .collect::<Vec<_>>();
        let identities = self.identities.resolve(adb_path, &serials);

        failed
            .into_iter()
            .filter_map(|result| {
                let identity = identities.get(&result.device_id)?;
                let adapter = self.recognize(identity)?;
                let hint =
                    adapter.explain(identity, &DeviceFailure::new(action, &result.message))?;

                Some(DeviceHint {
                    device_id: result.device_id.clone(),
                    vendor_id: adapter.profile().id,
                    hint: hint.into(),
                    ..Default::default()
                })
            })
            .collect()
    }
}
