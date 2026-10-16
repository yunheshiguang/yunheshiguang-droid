use yunheshiguang_sdk::contract::yunheshiguang::automation::v1::{Flow, FlowStep};
use yunheshiguang_sdk::contract::yunheshiguang::packages::v1::{
    PackageFlow, PackageManifest, PackageScript,
};
use yunheshiguang_sdk::contract::yunheshiguang::scripts::v1::{Script, ScriptLanguage};
use yunheshiguang_sdk::contract::yunheshiguang::templates::v1::{
    FlowTemplate, TemplateSourceInfo, TemplateStep,
};
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::BatchActionKind;
use yunheshiguang_sdk::templates::{TemplatePackage, TemplateSource};
use yunheshiguang_sdk::text::ui_text;

use crate::messages::*;

pub const MARKET_SOURCE_ID: &str = "reference.market";
pub const MARKET_CAPABILITY: &str = "reference.market";
pub const NIGHTLY_PACKAGE_ID: &str = "reference.nightly-check";
const BATTERY_SCRIPT_ID: &str = "reference-battery-report";

fn nightly_package() -> TemplatePackage {
    let step = |id: &str, kind: BatchActionKind, value: &str| FlowStep {
        id: id.to_string(),
        kind: kind.into(),
        value: value.to_string(),
        ..Default::default()
    };

    TemplatePackage {
        manifest: PackageManifest {
            id: NIGHTLY_PACKAGE_ID.to_string(),
            name: "夜间巡检 Nightly check".to_string(),
            version: "1.0.0".to_string(),
            author: "Reference extension".to_string(),
            description: "Reconnects dropped devices, then reports battery and Android version."
                .to_string(),
            created_at: Default::default(),
            ..Default::default()
        }
        .into(),
        flows: vec![PackageFlow {
            id: "reference-nightly-check".to_string(),
            flow: Flow {
                name: "夜间巡检 Nightly check".to_string(),
                steps: vec![
                    step("reconnect", BatchActionKind::RecoverConnection, ""),
                    step("report", BatchActionKind::RunScript, BATTERY_SCRIPT_ID),
                ],
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }],
        scripts: vec![PackageScript {
            id: BATTERY_SCRIPT_ID.to_string(),
            script: Script {
                name: "电量报告 Battery report".to_string(),
                language: ScriptLanguage::Shell.into(),
                body: "dumpsys battery | grep -E 'level|status'\ngetprop ro.build.version.release"
                    .to_string(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }],
        ..TemplatePackage::default()
    }
}

pub struct MarketTemplates;

fn template(
    id: &str,
    name_key: &str,
    description_key: &str,
    steps: &[BatchActionKind],
) -> FlowTemplate {
    FlowTemplate {
        id: id.to_string(),
        source_id: String::new(),
        name: ui_text(name_key).into(),
        description: ui_text(description_key).into(),
        steps: steps
            .iter()
            .map(|kind| TemplateStep {
                kind: (*kind).into(),
                value: String::new(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

impl TemplateSource for MarketTemplates {
    fn describe(&self) -> TemplateSourceInfo {
        TemplateSourceInfo {
            id: MARKET_SOURCE_ID.to_string(),
            label: ui_text(REFERENCE_MARKET_LABEL_KEY).into(),
            description: ui_text(REFERENCE_MARKET_DESCRIPTION_KEY).into(),
            ..Default::default()
        }
    }

    fn required_capability(&self) -> Option<&str> {
        Some(MARKET_CAPABILITY)
    }

    fn templates(&self) -> Vec<FlowTemplate> {
        vec![
            template(
                "nightly-health-check",
                REFERENCE_MARKET_NIGHTLY_NAME_KEY,
                REFERENCE_MARKET_NIGHTLY_DESCRIPTION_KEY,
                &[
                    BatchActionKind::RecoverConnection,
                    BatchActionKind::Screenshot,
                ],
            ),
            template(
                "release-smoke-test",
                REFERENCE_MARKET_RELEASE_NAME_KEY,
                REFERENCE_MARKET_RELEASE_DESCRIPTION_KEY,
                &[
                    BatchActionKind::InstallApk,
                    BatchActionKind::LaunchApp,
                    BatchActionKind::Screenshot,
                ],
            ),
        ]
    }

    fn packages(&self) -> Vec<TemplatePackage> {
        vec![nightly_package()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_its_templates_behind_the_market_capability() {
        let templates = MarketTemplates.templates();

        assert_eq!(
            MarketTemplates.required_capability(),
            Some(MARKET_CAPABILITY)
        );
        assert_eq!(MarketTemplates.describe().id, MARKET_SOURCE_ID);
        assert_eq!(
            templates
                .iter()
                .map(|template| (template.id.as_str(), template.steps.len()))
                .collect::<Vec<_>>(),
            vec![("nightly-health-check", 2), ("release-smoke-test", 3)]
        );
    }

    #[test]
    fn sells_a_package_that_brings_the_scripts_its_flow_runs() {
        let packages = MarketTemplates.packages();
        let package = &packages[0];
        let script_ids = package
            .scripts
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(package.manifest.as_option().unwrap().id, NIGHTLY_PACKAGE_ID);
        for step in &package.flows[0].flow.as_option().unwrap().steps {
            if step.kind == BatchActionKind::RunScript {
                assert!(script_ids.contains(&step.value.as_str()));
            }
        }
    }
}
