use yunheshiguang_contract::yunheshiguang::templates::v1::{
    FlowTemplate, TemplateSourceInfo, TemplateStep,
};
use yunheshiguang_sdk::templates::TemplateSource;

use crate::workspace::contract::BatchActionKind;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

pub const STARTER_SOURCE_ID: &str = "core.starter";

pub struct StarterTemplates;

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

impl TemplateSource for StarterTemplates {
    fn describe(&self) -> TemplateSourceInfo {
        TemplateSourceInfo {
            id: STARTER_SOURCE_ID.to_string(),
            label: ui_text(TEMPLATES_STARTER_LABEL_KEY).into(),
            description: ui_text(TEMPLATES_STARTER_DESCRIPTION_KEY).into(),
            ..Default::default()
        }
    }

    fn templates(&self) -> Vec<FlowTemplate> {
        vec![
            template(
                "install-and-launch",
                TEMPLATES_INSTALL_AND_LAUNCH_NAME_KEY,
                TEMPLATES_INSTALL_AND_LAUNCH_DESCRIPTION_KEY,
                &[BatchActionKind::InstallApk, BatchActionKind::LaunchApp],
            ),
            template(
                "capture-screens",
                TEMPLATES_CAPTURE_SCREENS_NAME_KEY,
                TEMPLATES_CAPTURE_SCREENS_DESCRIPTION_KEY,
                &[BatchActionKind::Screenshot],
            ),
            template(
                "recover-and-capture",
                TEMPLATES_RECOVER_AND_CAPTURE_NAME_KEY,
                TEMPLATES_RECOVER_AND_CAPTURE_DESCRIPTION_KEY,
                &[
                    BatchActionKind::RecoverConnection,
                    BatchActionKind::Screenshot,
                ],
            ),
        ]
    }
}
