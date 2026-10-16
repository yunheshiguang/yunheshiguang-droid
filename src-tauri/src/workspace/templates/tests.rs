use yunheshiguang_contract::yunheshiguang::templates::v1::{
    FlowTemplate, TemplateSourceInfo, TemplateStep,
};
use yunheshiguang_sdk::templates::TemplateSource;

use super::catalog::TemplateCatalog;
use super::starter::STARTER_SOURCE_ID;
use crate::workspace::contract::BatchActionKind;

struct Market;

impl TemplateSource for Market {
    fn describe(&self) -> TemplateSourceInfo {
        TemplateSourceInfo {
            id: "test.market".to_string(),
            ..TemplateSourceInfo::default()
        }
    }

    fn required_capability(&self) -> Option<&str> {
        Some("market.templates")
    }

    fn templates(&self) -> Vec<FlowTemplate> {
        vec![FlowTemplate {
            id: "night-report".to_string(),
            source_id: "core.starter".to_string(),
            steps: vec![TemplateStep {
                kind: BatchActionKind::Screenshot.into(),
                value: "/reports".to_string(),
                ..Default::default()
            }],
            ..FlowTemplate::default()
        }]
    }
}

#[test]
fn offers_the_starter_templates_with_real_actions() {
    let catalog = TemplateCatalog::new(Vec::new()).catalog(&[]);

    assert_eq!(catalog.sources.len(), 1);
    assert_eq!(catalog.sources[0].id, STARTER_SOURCE_ID);
    assert!(catalog.templates.len() >= 3);

    for template in &catalog.templates {
        assert_eq!(template.source_id, STARTER_SOURCE_ID);
        assert!(!template.steps.is_empty(), "{} has no steps", template.id);
        assert!(template.name.is_set() && template.description.is_set());

        for step in &template.steps {
            assert_ne!(
                step.kind,
                BatchActionKind::Unspecified,
                "{} has a step without an action",
                template.id
            );
        }
    }
}

#[test]
fn keeps_a_source_hidden_until_its_capability_is_granted() {
    let catalog = TemplateCatalog::new(vec![Box::new(Market)]);

    let locked = catalog.catalog(&["other".to_string()]);
    assert!(
        locked
            .sources
            .iter()
            .all(|source| source.id != "test.market")
    );
    assert!(
        locked
            .templates
            .iter()
            .all(|template| template.id != "night-report")
    );

    let unlocked = catalog.catalog(&["market.templates".to_string()]);
    let ids: Vec<_> = unlocked
        .sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();
    assert_eq!(ids, [STARTER_SOURCE_ID, "test.market"]);

    let night = unlocked
        .templates
        .iter()
        .find(|template| template.id == "night-report")
        .unwrap();
    assert_eq!(
        night.source_id, "test.market",
        "the catalog stamps the source"
    );
}
