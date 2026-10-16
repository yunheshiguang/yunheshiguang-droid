use crate::contract::yunheshiguang::vendors::v1::VendorProfile;
use crate::contract::yunheshiguang::workspace::v1::{BatchActionKind, UiText};

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub manufacturer: String,
    pub brand: String,
    pub model: String,
    pub product: String,
    pub device: String,
    pub platform: String,
    pub serial_number: String,
    pub sdk_version: String,
}

impl DeviceIdentity {
    pub fn from_properties<'a>(properties: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut identity = Self::default();
        let mut boot_serial = String::new();

        for (key, value) in properties {
            let slot = match key {
                "ro.product.manufacturer" => &mut identity.manufacturer,
                "ro.product.brand" => &mut identity.brand,
                "ro.product.model" => &mut identity.model,
                "ro.product.name" => &mut identity.product,
                "ro.product.device" => &mut identity.device,
                "ro.board.platform" => &mut identity.platform,
                "ro.serialno" => &mut identity.serial_number,
                "ro.boot.serialno" => &mut boot_serial,
                "ro.build.version.sdk" => &mut identity.sdk_version,
                _ => continue,
            };
            *slot = value.trim().to_string();
        }

        if identity.serial_number.is_empty() {
            identity.serial_number = boot_serial;
        }

        identity
    }

    pub fn is_brand(&self, names: &[&str]) -> bool {
        names.iter().any(|name| {
            self.brand.eq_ignore_ascii_case(name) || self.manufacturer.eq_ignore_ascii_case(name)
        })
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub struct DeviceFailure<'a> {
    pub action: BatchActionKind,
    pub message: &'a str,
}

impl<'a> DeviceFailure<'a> {
    pub fn new(action: BatchActionKind, message: &'a str) -> Self {
        Self { action, message }
    }

    pub fn mentions(&self, phrase: &str) -> bool {
        self.message
            .to_ascii_lowercase()
            .contains(&phrase.to_ascii_lowercase())
    }
}

pub trait VendorAdapter: Send + Sync {
    fn profile(&self) -> VendorProfile;

    fn recognizes(&self, device: &DeviceIdentity) -> bool;

    fn explain(&self, _device: &DeviceIdentity, _failure: &DeviceFailure<'_>) -> Option<UiText> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_the_properties_it_knows() {
        let identity = DeviceIdentity::from_properties([
            ("ro.product.brand", "Redmi "),
            ("ro.product.manufacturer", "Xiaomi"),
            ("ro.board.platform", "taro"),
            ("ro.build.version.sdk", "34"),
            ("persist.sys.locale", "zh-CN"),
        ]);

        assert_eq!(identity.brand, "Redmi");
        assert_eq!(identity.platform, "taro");
        assert_eq!(identity.sdk_version, "34");
        assert!(identity.is_brand(&["xiaomi"]));
        assert!(identity.is_brand(&["REDMI"]));
        assert!(!identity.is_brand(&["huawei"]));
    }

    #[test]
    fn falls_back_to_the_boot_serial_whichever_order_getprop_lists_them() {
        let boot_first = DeviceIdentity::from_properties([
            ("ro.boot.serialno", "BOOT123"),
            ("ro.serialno", "R58M20ABCDE"),
        ]);
        let boot_only = DeviceIdentity::from_properties([("ro.boot.serialno", "BOOT123")]);

        assert_eq!(boot_first.serial_number, "R58M20ABCDE");
        assert_eq!(boot_only.serial_number, "BOOT123");
    }

    #[test]
    fn matches_failure_text_without_caring_about_case() {
        let failure = DeviceFailure::new(
            BatchActionKind::InstallApk,
            "Failure [INSTALL_FAILED_USER_RESTRICTED: Install canceled by user]",
        );

        assert!(failure.mentions("install_failed_user_restricted"));
        assert!(!failure.mentions("INJECT_EVENTS"));
    }
}
