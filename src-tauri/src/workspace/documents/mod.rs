mod module;
mod service;
pub mod store;

pub use module::DocumentsModule;
pub use service::{DOCUMENTS_CHANGED_EVENT, DocumentsError, DocumentsService};

mod rpc;
#[cfg(test)]
mod tests;
