use yunheshiguang_contract::KnownEnum;

mod actions;
mod devices;
mod errors;
mod executor;
mod outcome;
mod policy;
mod report;
mod service;
mod spec;
mod task;

use crate::workspace::adb::AdbDeviceState;
use crate::workspace::contract::BatchActionKind;

pub use service::{BatchModule, BatchService};
pub use spec::{fill_request, step_problem, uses_host_files};

pub fn reaches_offline(kind: i32) -> bool {
    actions::handler(BatchActionKind::known(kind).unwrap_or(BatchActionKind::Unspecified))
        .allowed_states()
        .contains(&AdbDeviceState::Offline)
}

mod rpc;
#[cfg(test)]
mod tests;
