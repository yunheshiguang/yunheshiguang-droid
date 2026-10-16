mod catalog;
mod module;
mod starter;

pub use catalog::TemplateCatalog;
pub use module::TemplatesModule;

mod rpc;
#[cfg(test)]
mod tests;
