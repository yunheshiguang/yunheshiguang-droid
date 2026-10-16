use crate::workspace::contract::AndroidPlatformToolsInstallStatus;

use super::record::AndroidPlatformToolsStateRecord;

pub const STATE_SCHEMA_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationRejection {
    NewerSchema { found: u32 },
}

impl MigrationRejection {
    pub fn describe(&self) -> String {
        match self {
            Self::NewerSchema { found } => format!(
                "the settings file uses schema version {found}, but this build understands \
                 version {STATE_SCHEMA_VERSION}"
            ),
        }
    }
}

pub fn migrate(
    mut state: AndroidPlatformToolsStateRecord,
) -> Result<AndroidPlatformToolsStateRecord, MigrationRejection> {
    if state.schema_version > STATE_SCHEMA_VERSION {
        return Err(MigrationRejection::NewerSchema {
            found: state.schema_version,
        });
    }

    if state.global_config.is_none() {
        state.global_config = state
            .window_configs
            .get("main")
            .cloned()
            .or_else(|| state.window_configs.values().next().cloned());
    }
    state.window_configs.clear();

    state.schema_version = STATE_SCHEMA_VERSION;

    Ok(state)
}

pub fn settle_interrupted_install(
    mut state: AndroidPlatformToolsStateRecord,
) -> AndroidPlatformToolsStateRecord {
    let installing = AndroidPlatformToolsInstallStatus::Installing;

    if state.last_attempt.status() == installing {
        state.last_attempt = state.last_attempt.interrupted();
    }

    if state.install.status() == installing {
        state.last_attempt = state.install.clone().interrupted();
        state.install = Default::default();
    }

    state
}

pub fn default_state() -> AndroidPlatformToolsStateRecord {
    AndroidPlatformToolsStateRecord {
        schema_version: STATE_SCHEMA_VERSION,
        ..AndroidPlatformToolsStateRecord::default()
    }
}
