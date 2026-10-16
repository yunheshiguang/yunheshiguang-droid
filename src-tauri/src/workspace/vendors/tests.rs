use std::path::Path;
use std::sync::Arc;

use yunheshiguang_sdk::vendor::{DeviceIdentity, VendorAdapter};

use super::registry::VendorRegistry;
use crate::workspace::adb::DeviceIdentities;
use crate::workspace::contract::{AdbDeviceActionResult, BatchActionKind, VendorProfile};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

const NO_ADB: &str = "/nonexistent/adb";

fn identity(brand: &str, platform: &str) -> DeviceIdentity {
    DeviceIdentity::from_properties([("ro.product.brand", brand), ("ro.board.platform", platform)])
}

fn registry(
    adapters: Vec<Box<dyn VendorAdapter>>,
    devices: &[(&str, DeviceIdentity)],
) -> VendorRegistry {
    let identities = Arc::new(DeviceIdentities::default());

    for (serial, identity) in devices {
        identities.remember(serial, identity.clone());
    }

    VendorRegistry::new(adapters, identities)
}

fn failed(device_id: &str, message: &str) -> AdbDeviceActionResult {
    AdbDeviceActionResult {
        device_id: device_id.to_string(),
        success: false,
        message: message.to_string(),
        ..AdbDeviceActionResult::default()
    }
}

#[test]
fn recognizes_the_common_brands_and_leaves_the_rest_alone() {
    let vendors = registry(
        Vec::new(),
        &[
            ("redmi", identity("Redmi", "taro")),
            ("honor", identity("HONOR", "kirin")),
            ("pixel", identity("google", "zuma")),
        ],
    );

    let result = vendors.vendors(
        Path::new(NO_ADB),
        &[
            "redmi".to_string(),
            "honor".to_string(),
            "pixel".to_string(),
            "offline".to_string(),
        ],
    );

    assert_eq!(
        result
            .devices
            .iter()
            .map(|device| (device.device_id.as_str(), device.vendor_id.as_str()))
            .collect::<Vec<_>>(),
        vec![("redmi", "core.xiaomi"), ("honor", "core.huawei")]
    );
    assert_eq!(result.vendors.len(), 2);
    assert!(!result.vendors[0].guidance.is_empty());
}

struct BoardFarm;

impl VendorAdapter for BoardFarm {
    fn profile(&self) -> VendorProfile {
        VendorProfile {
            id: "farm".to_string(),
            ..VendorProfile::default()
        }
    }

    fn recognizes(&self, device: &DeviceIdentity) -> bool {
        device.platform.starts_with("rk")
    }
}

#[test]
fn lets_an_extension_adapter_speak_before_the_built_in_brands() {
    let vendors = registry(
        vec![Box::new(BoardFarm)],
        &[("board", identity("Xiaomi", "rk3588"))],
    );

    let result = vendors.vendors(Path::new(NO_ADB), &["board".to_string()]);

    assert_eq!(result.devices[0].vendor_id, "farm");
}

#[test]
fn explains_known_failures_on_recognized_devices_only() {
    let vendors = registry(
        Vec::new(),
        &[
            ("redmi", identity("Redmi", "taro")),
            ("pixel", identity("google", "zuma")),
        ],
    );

    let hints = vendors.hints(
        Path::new(NO_ADB),
        BatchActionKind::InstallApk,
        &[
            failed(
                "redmi",
                "Failure [INSTALL_FAILED_USER_RESTRICTED: Install canceled by user]",
            ),
            failed("pixel", "Failure [INSTALL_FAILED_USER_RESTRICTED]"),
            AdbDeviceActionResult {
                device_id: "redmi-ok".to_string(),
                success: true,
                ..AdbDeviceActionResult::default()
            },
        ],
    );

    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].device_id, "redmi");
    assert_eq!(hints[0].vendor_id, "core.xiaomi");
    assert_eq!(
        hints[0].hint,
        ui_text(VENDOR_XIAOMI_EXPLAIN_INSTALL_VIA_USB_KEY).into()
    );
    assert!(
        vendors
            .hints(
                Path::new(NO_ADB),
                BatchActionKind::InputText,
                &[failed("redmi", "timed out")],
            )
            .is_empty()
    );
}
