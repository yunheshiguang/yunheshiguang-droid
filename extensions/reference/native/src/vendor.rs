use std::path::Path;

use serde::Deserialize;
use yunheshiguang_sdk::contract::yunheshiguang::vendors::v1::VendorProfile;
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::UiText;
use yunheshiguang_sdk::text::ui_text;
use yunheshiguang_sdk::vendor::{DeviceFailure, DeviceIdentity, VendorAdapter};

use crate::config::read_json;
use crate::messages::*;
use crate::status::FileState;

const DEFAULT_BOARD_PLATFORMS: &[&str] = &["rk3288", "rk3399", "rk3566", "rk3568", "rk3588"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct VendorRules {
    board_platforms: Vec<String>,
}

pub struct BoardFarmVendor {
    platforms: Vec<String>,
    state: FileState,
}

impl BoardFarmVendor {
    pub fn load(path: &Path) -> Self {
        let defaults = || {
            DEFAULT_BOARD_PLATFORMS
                .iter()
                .map(|platform| (*platform).to_string())
                .collect()
        };

        match read_json::<VendorRules>(path) {
            Ok(Some(rules)) => Self {
                platforms: rules.board_platforms,
                state: FileState::Loaded,
            },
            Ok(None) => Self {
                platforms: defaults(),
                state: FileState::Missing,
            },
            Err(reason) => Self {
                platforms: defaults(),
                state: FileState::Broken(reason),
            },
        }
    }

    pub fn state(&self) -> &FileState {
        &self.state
    }
}

impl VendorAdapter for BoardFarmVendor {
    fn profile(&self) -> VendorProfile {
        VendorProfile {
            id: "reference.board-farm".to_string(),
            label: ui_text(REFERENCE_BOARD_FARM_LABEL_KEY).into(),
            guidance: vec![
                ui_text(REFERENCE_BOARD_FARM_POWER_KEY),
                ui_text(REFERENCE_BOARD_FARM_ADB_KEY_KEY),
            ],
            ..Default::default()
        }
    }

    fn recognizes(&self, device: &DeviceIdentity) -> bool {
        self.platforms
            .iter()
            .any(|platform| device.platform.eq_ignore_ascii_case(platform))
    }

    fn explain(&self, _device: &DeviceIdentity, failure: &DeviceFailure<'_>) -> Option<UiText> {
        failure
            .mentions("INSTALL_FAILED_INSUFFICIENT_STORAGE")
            .then(|| ui_text(REFERENCE_BOARD_FARM_STORAGE_KEY))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::BatchActionKind;

    use super::*;

    fn board(platform: &str) -> DeviceIdentity {
        DeviceIdentity::from_properties([("ro.board.platform", platform)])
    }

    #[test]
    fn recognizes_common_board_platforms_without_a_rules_file() {
        let folder = tempfile::tempdir().unwrap();
        let vendor = BoardFarmVendor::load(&folder.path().join("vendors.json"));

        assert_eq!(vendor.state(), &FileState::Missing);
        assert!(vendor.recognizes(&board("RK3588")));
        assert!(!vendor.recognizes(&board("taro")));
    }

    #[test]
    fn follows_the_rules_file_and_explains_a_full_data_partition() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("vendors.json");
        fs::write(&path, r#"{ "boardPlatforms": ["sm8150"] }"#).unwrap();
        let vendor = BoardFarmVendor::load(&path);
        let failure = DeviceFailure::new(
            BatchActionKind::InstallApk,
            "Failure [INSTALL_FAILED_INSUFFICIENT_STORAGE]",
        );

        assert!(vendor.recognizes(&board("sm8150")));
        assert!(!vendor.recognizes(&board("rk3588")));
        assert_eq!(
            vendor.explain(&board("sm8150"), &failure),
            Some(ui_text(REFERENCE_BOARD_FARM_STORAGE_KEY))
        );
    }
}
