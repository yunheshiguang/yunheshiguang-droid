pub use crate::contract::yunheshiguang::packages::v1::TemplatePackage;
use crate::contract::yunheshiguang::templates::v1::{FlowTemplate, TemplateSourceInfo};

pub trait TemplateSource: Send + Sync {
    fn describe(&self) -> TemplateSourceInfo;

    fn required_capability(&self) -> Option<&str> {
        None
    }

    fn templates(&self) -> Vec<FlowTemplate>;

    fn packages(&self) -> Vec<TemplatePackage> {
        Vec::new()
    }
}
