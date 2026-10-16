use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use yunheshiguang_sdk::contract::wkt::Timestamp;
use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::{
    GovernanceProvider, License, LicenseStatus,
};
use yunheshiguang_sdk::governance::{EntitlementProvider, provider_info};
use yunheshiguang_sdk::text::ui_text_with_params;

use crate::config::read_json;
use crate::messages::*;
use crate::status::FileState;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LicenseFile {
    pub plan_id: String,
    pub plan_name: String,
    pub licensee: String,
    #[serde(default)]
    pub trial: bool,
    #[serde(default)]
    pub expires_at: Option<Timestamp>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

pub struct LicenseEntitlements {
    license: Option<LicenseFile>,
    state: FileState,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

impl LicenseEntitlements {
    pub fn load(path: &Path) -> Self {
        match read_json::<LicenseFile>(path) {
            Ok(Some(license)) => Self::new(Some(license), FileState::Loaded),
            Ok(None) => Self::new(None, FileState::Missing),
            Err(reason) => {
                log::error!("the license at {} is unusable: {reason}", path.display());
                Self::new(None, FileState::Broken(reason))
            }
        }
    }

    fn new(license: Option<LicenseFile>, state: FileState) -> Self {
        Self { license, state }
    }

    pub fn state(&self) -> &FileState {
        &self.state
    }

    fn status_at(license: &LicenseFile, now: i64) -> LicenseStatus {
        match &license.expires_at {
            Some(expires_at) if expires_at.seconds <= now => LicenseStatus::Expired,
            _ if license.trial => LicenseStatus::Trial,
            _ => LicenseStatus::Active,
        }
    }

    pub fn capabilities_at(&self, now: i64) -> Vec<String> {
        match &self.license {
            Some(license) if Self::status_at(license, now) != LicenseStatus::Expired => {
                license.capabilities.clone()
            }
            _ => Vec::new(),
        }
    }

    pub fn license_at(&self, now: i64) -> Option<License> {
        self.license.as_ref().map(|license| License {
            plan_id: license.plan_id.clone(),
            plan: ui_text_with_params(
                REFERENCE_LICENSE_PLAN_KEY,
                license.plan_name.as_str(),
                [("plan", license.plan_name.as_str())],
            )
            .into(),
            status: Self::status_at(license, now).into(),
            expires_at: license.expires_at.clone().into(),
            licensee: license.licensee.clone(),
            ..Default::default()
        })
    }
}

impl EntitlementProvider for LicenseEntitlements {
    fn describe(&self) -> GovernanceProvider {
        provider_info(
            "reference.license-file",
            REFERENCE_LICENSE_LABEL_KEY,
            REFERENCE_LICENSE_DESCRIPTION_KEY,
        )
    }

    fn granted_capabilities(&self) -> Vec<String> {
        self.capabilities_at(unix_now())
    }

    fn license(&self) -> Option<License> {
        self.license_at(unix_now())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const EXPIRES_AT: i64 = 1_798_761_600;

    fn load(contents: &str) -> LicenseEntitlements {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("license.json");
        fs::write(&path, contents).unwrap();

        LicenseEntitlements::load(&path)
    }

    fn enterprise(trial: bool) -> LicenseEntitlements {
        load(&format!(
            r#"{{
                "planId": "enterprise",
                "planName": "Enterprise",
                "licensee": "Example Co.",
                "trial": {trial},
                "expiresAt": "2027-01-01T00:00:00Z",
                "capabilities": ["reference.market", "reference.enterprise"]
            }}"#
        ))
    }

    #[test]
    fn grants_nothing_and_shows_no_license_without_a_file() {
        let folder = tempfile::tempdir().unwrap();
        let entitlements = LicenseEntitlements::load(&folder.path().join("license.json"));

        assert_eq!(entitlements.state(), &FileState::Missing);
        assert!(entitlements.granted_capabilities().is_empty());
        assert_eq!(entitlements.license(), None);
    }

    #[test]
    fn grants_the_licensed_capabilities_until_the_license_expires() {
        let entitlements = enterprise(false);
        let license = entitlements.license_at(EXPIRES_AT - 1).unwrap();

        assert_eq!(entitlements.state(), &FileState::Loaded);
        assert_eq!(
            entitlements.capabilities_at(EXPIRES_AT - 1),
            vec!["reference.market", "reference.enterprise"]
        );
        assert_eq!(license.status, LicenseStatus::Active);
        assert_eq!(license.plan_id, "enterprise");
        assert_eq!(license.licensee, "Example Co.");
        assert_eq!(license.expires_at.unwrap().seconds, EXPIRES_AT);
        assert_eq!(license.plan.unwrap().params["plan"], "Enterprise");

        assert!(entitlements.capabilities_at(EXPIRES_AT).is_empty());
        assert_eq!(
            entitlements.license_at(EXPIRES_AT).unwrap().status,
            LicenseStatus::Expired
        );
    }

    #[test]
    fn marks_trial_licenses() {
        assert_eq!(
            enterprise(true).license_at(0).unwrap().status,
            LicenseStatus::Trial
        );
    }

    #[test]
    fn treats_a_broken_file_as_no_license_and_says_why() {
        let entitlements = load(r#"{ "planId": "enterprise" }"#);

        assert!(
            matches!(entitlements.state(), FileState::Broken(reason) if reason.contains("planName"))
        );
        assert!(entitlements.capabilities_at(0).is_empty());
        assert_eq!(entitlements.license_at(0), None);
    }
}
