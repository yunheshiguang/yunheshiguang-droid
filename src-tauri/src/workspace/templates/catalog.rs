use yunheshiguang_contract::yunheshiguang::templates::v1::{
    FlowTemplateCatalog, TemplateSourceInfo,
};
use yunheshiguang_sdk::templates::{TemplatePackage, TemplateSource};

use super::starter::StarterTemplates;

pub struct TemplateCatalog {
    sources: Vec<Box<dyn TemplateSource>>,
}

impl TemplateCatalog {
    pub fn new(contributed: Vec<Box<dyn TemplateSource>>) -> Self {
        let mut sources: Vec<Box<dyn TemplateSource>> = vec![Box::new(StarterTemplates)];
        sources.extend(contributed);

        Self { sources }
    }

    fn unlocked<'a>(
        &'a self,
        granted: &'a [String],
    ) -> impl Iterator<Item = &'a dyn TemplateSource> + 'a {
        self.sources
            .iter()
            .map(AsRef::as_ref)
            .filter(move |source| {
                source
                    .required_capability()
                    .is_none_or(|capability| granted.iter().any(|key| key == capability))
            })
    }

    pub fn catalog(&self, granted: &[String]) -> FlowTemplateCatalog {
        let mut catalog = FlowTemplateCatalog::default();

        for source in self.unlocked(granted) {
            let info = source.describe();

            catalog
                .templates
                .extend(source.templates().into_iter().map(|mut template| {
                    template.source_id = info.id.clone();
                    template
                }));
            catalog.sources.push(info);
        }

        catalog
    }

    pub fn packages(&self, granted: &[String]) -> Vec<(TemplateSourceInfo, TemplatePackage)> {
        self.unlocked(granted)
            .flat_map(|source| {
                let info = source.describe();
                source
                    .packages()
                    .into_iter()
                    .map(move |package| (info.clone(), package))
            })
            .collect()
    }
}
