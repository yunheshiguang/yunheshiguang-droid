mod builtin;
mod module;
mod registry;

pub use module::VendorsModule;
pub use registry::VendorRegistry;

mod rpc;
#[cfg(test)]
mod tests;
