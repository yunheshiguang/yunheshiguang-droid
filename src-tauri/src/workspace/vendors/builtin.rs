use yunheshiguang_sdk::vendor::{DeviceFailure, DeviceIdentity, VendorAdapter};

use crate::workspace::contract::{UiText, VendorProfile};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

#[derive(Clone, Copy)]
struct Explanation {
    phrases: &'static [&'static str],
    hint: &'static str,
}

#[derive(Clone, Copy)]
pub struct BrandAdapter {
    id: &'static str,
    brands: &'static [&'static str],
    label: &'static str,
    guidance: &'static [&'static str],
    explanations: &'static [Explanation],
}

const INPUT_BLOCKED: &[&str] = &["INJECT_EVENTS", "SecurityException"];

const BRANDS: &[BrandAdapter] = &[
    BrandAdapter {
        id: "core.xiaomi",
        brands: &["xiaomi", "redmi", "poco"],
        label: VENDOR_XIAOMI_LABEL_KEY,
        guidance: &[
            VENDOR_XIAOMI_SECURITY_SETTINGS_KEY,
            VENDOR_XIAOMI_INSTALL_VIA_USB_KEY,
        ],
        explanations: &[
            Explanation {
                phrases: INPUT_BLOCKED,
                hint: VENDOR_XIAOMI_EXPLAIN_SECURITY_SETTINGS_KEY,
            },
            Explanation {
                phrases: &["INSTALL_FAILED_USER_RESTRICTED"],
                hint: VENDOR_XIAOMI_EXPLAIN_INSTALL_VIA_USB_KEY,
            },
        ],
    },
    BrandAdapter {
        id: "core.huawei",
        brands: &["huawei", "honor"],
        label: VENDOR_HUAWEI_LABEL_KEY,
        guidance: &[
            VENDOR_HUAWEI_CHARGE_ONLY_KEY,
            VENDOR_HUAWEI_ALWAYS_ALLOW_KEY,
        ],
        explanations: &[],
    },
    BrandAdapter {
        id: "core.oppo",
        brands: &["oppo", "oneplus", "realme"],
        label: VENDOR_OPPO_LABEL_KEY,
        guidance: &[
            VENDOR_OPPO_PERMISSION_MONITORING_KEY,
            VENDOR_OPPO_INSTALL_PROMPT_KEY,
        ],
        explanations: &[
            Explanation {
                phrases: INPUT_BLOCKED,
                hint: VENDOR_OPPO_EXPLAIN_PERMISSION_MONITORING_KEY,
            },
            Explanation {
                phrases: &[
                    "INSTALL_FAILED_ABORTED",
                    "INSTALL_FAILED_VERIFICATION_FAILURE",
                ],
                hint: VENDOR_OPPO_EXPLAIN_INSTALL_PROMPT_KEY,
            },
        ],
    },
    BrandAdapter {
        id: "core.samsung",
        brands: &["samsung"],
        label: VENDOR_SAMSUNG_LABEL_KEY,
        guidance: &[VENDOR_SAMSUNG_AUTO_BLOCKER_KEY],
        explanations: &[],
    },
];

impl VendorAdapter for BrandAdapter {
    fn profile(&self) -> VendorProfile {
        VendorProfile {
            id: self.id.to_string(),
            label: ui_text(self.label).into(),
            guidance: self.guidance.iter().map(|key| ui_text(*key)).collect(),
            ..Default::default()
        }
    }

    fn recognizes(&self, device: &DeviceIdentity) -> bool {
        device.is_brand(self.brands)
    }

    fn explain(&self, _device: &DeviceIdentity, failure: &DeviceFailure<'_>) -> Option<UiText> {
        self.explanations
            .iter()
            .find(|explanation| {
                explanation
                    .phrases
                    .iter()
                    .any(|phrase| failure.mentions(phrase))
            })
            .map(|explanation| ui_text(explanation.hint))
    }
}

pub fn builtin_adapters() -> impl Iterator<Item = Box<dyn VendorAdapter>> {
    BRANDS
        .iter()
        .map(|adapter| Box::new(*adapter) as Box<dyn VendorAdapter>)
}
