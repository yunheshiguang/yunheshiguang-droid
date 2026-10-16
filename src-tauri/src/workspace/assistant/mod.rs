mod drafts;
mod module;
mod openai;
mod prompts;
mod service;
mod settings;

pub use module::AssistantModule;
pub use service::AssistantService;

mod rpc;
#[cfg(test)]
mod tests;
