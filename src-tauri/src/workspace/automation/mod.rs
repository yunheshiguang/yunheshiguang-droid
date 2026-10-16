mod flows;
mod history;
mod module;
mod options;
mod runner;
mod scheduler;
mod schedules;
mod service;
mod steps;
mod targets;

pub use flows::{FLOWS_COLLECTION, list_flows};
pub use module::AutomationModule;
pub use runner::AUTOMATION_CHANGED_EVENT;
pub use service::AutomationService;

mod rpc;
#[cfg(test)]
mod tests;
