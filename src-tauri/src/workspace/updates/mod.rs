mod build_info;
mod checker;
mod install;
mod manifest;
mod module;
mod rpc;
#[cfg(test)]
mod tests;

pub use module::UpdatesModule;

pub const UPDATE_AVAILABLE_EVENT: &str = "workspace:update-available";
