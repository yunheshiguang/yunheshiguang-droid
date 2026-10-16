use log::LevelFilter;
use tauri::Runtime;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

const MAX_LOG_FILE_BYTES: u128 = 2 * 1024 * 1024;
const KEPT_LOG_FILES: usize = 3;

pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let own_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
        ])
        .level(LevelFilter::Info)
        .level_for(env!("CARGO_CRATE_NAME"), own_level)
        .max_file_size(MAX_LOG_FILE_BYTES)
        .rotation_strategy(RotationStrategy::KeepSome(KEPT_LOG_FILES))
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .build()
}
