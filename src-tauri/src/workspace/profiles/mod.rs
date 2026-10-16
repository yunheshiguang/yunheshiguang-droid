mod book;
mod groups;
mod keys;
mod module;
mod patch;
mod section;
mod service;

pub use book::{Inventory, ProfileBook};
pub use keys::connected_device_keys;
pub use module::ProfilesModule;
pub use section::ProfilesSection;
pub use service::{DEVICE_PROFILES_CHANGED_EVENT, ProfilesService};

mod rpc;
#[cfg(test)]
mod tests;
