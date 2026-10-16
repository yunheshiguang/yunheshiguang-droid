pub use super::logs::reveal_log_directory;
pub use super::rpc::{rpc, rpc_cancel};

#[macro_export]
macro_rules! workspace_invoke_handler {
    () => {{
        use $crate::workspace::commands::{reveal_log_directory, rpc, rpc_cancel};

        tauri::generate_handler![rpc, rpc_cancel, reveal_log_directory]
    }};
}
