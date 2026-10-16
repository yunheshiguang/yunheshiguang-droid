use crate::workspace::contract::{
    AdbDeviceStateContract, ConnectionHealth, DeviceStatus, DiagnosticCode, UiText,
};
use crate::workspace::messages::{
    DEVICE_NOTE_ADB_STATUS_KEY, DEVICE_NOTE_CONNECTED_KEY, DEVICE_NOTE_OFFLINE_KEY,
    DEVICE_NOTE_USB_UNAUTHORIZED_KEY,
};
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::device::AdbDeviceState;

#[derive(Clone, Copy, Debug)]
pub struct DeviceStateFacts {
    pub contract: AdbDeviceStateContract,
    pub status: DeviceStatus,
    pub health: ConnectionHealth,
    pub diagnostic: DiagnosticCode,
    pub note_key: Option<&'static str>,
}

impl AdbDeviceState {
    pub fn facts(&self) -> DeviceStateFacts {
        match self {
            Self::Online => DeviceStateFacts {
                contract: AdbDeviceStateContract::Online,
                status: DeviceStatus::Online,
                health: ConnectionHealth::Ready,
                diagnostic: DiagnosticCode::None,
                note_key: Some(DEVICE_NOTE_CONNECTED_KEY),
            },
            Self::Offline => DeviceStateFacts {
                contract: AdbDeviceStateContract::Offline,
                status: DeviceStatus::Offline,
                health: ConnectionHealth::Blocked,
                diagnostic: DiagnosticCode::DeviceOffline,
                note_key: Some(DEVICE_NOTE_OFFLINE_KEY),
            },
            Self::Unauthorized => DeviceStateFacts {
                contract: AdbDeviceStateContract::Unauthorized,
                status: DeviceStatus::Warning,
                health: ConnectionHealth::Blocked,
                diagnostic: DiagnosticCode::UsbUnauthorized,
                note_key: Some(DEVICE_NOTE_USB_UNAUTHORIZED_KEY),
            },
            Self::NoPermissions => DeviceStateFacts {
                contract: AdbDeviceStateContract::NoPermissions,
                status: DeviceStatus::Warning,
                health: ConnectionHealth::Blocked,
                diagnostic: DiagnosticCode::AdbNoPermissions,
                note_key: None,
            },
            Self::Recovery => DeviceStateFacts {
                contract: AdbDeviceStateContract::Recovery,
                status: DeviceStatus::Warning,
                health: ConnectionHealth::Degraded,
                diagnostic: DiagnosticCode::None,
                note_key: None,
            },
            Self::Sideload => DeviceStateFacts {
                contract: AdbDeviceStateContract::Sideload,
                status: DeviceStatus::Warning,
                health: ConnectionHealth::Degraded,
                diagnostic: DiagnosticCode::None,
                note_key: None,
            },
            Self::Bootloader => DeviceStateFacts {
                contract: AdbDeviceStateContract::Bootloader,
                status: DeviceStatus::Warning,
                health: ConnectionHealth::Degraded,
                diagnostic: DiagnosticCode::None,
                note_key: None,
            },
            Self::Unknown(state) => DeviceStateFacts {
                contract: AdbDeviceStateContract::Unknown,
                status: DeviceStatus::Warning,
                health: ConnectionHealth::Degraded,
                diagnostic: match state.as_str() {
                    "authorizing" => DiagnosticCode::UsbUnauthorized,
                    "connecting" => DiagnosticCode::DeviceOffline,
                    _ => DiagnosticCode::AdbConflict,
                },
                note_key: None,
            },
        }
    }

    pub fn note(&self) -> UiText {
        match self.facts().note_key {
            Some(key) => ui_text(key),
            None => ui_text_with_params(
                DEVICE_NOTE_ADB_STATUS_KEY,
                "",
                [("status", self.as_str().to_string())],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_unrecognized_states_fall_back_to_the_raw_adb_status() {
        assert_eq!(AdbDeviceState::Online.note().key, DEVICE_NOTE_CONNECTED_KEY);

        let unknown = AdbDeviceState::Unknown("authorizing".to_string());
        let note = unknown.note();

        assert_eq!(note.key, DEVICE_NOTE_ADB_STATUS_KEY);
        assert_eq!(
            note.params.get("status").map(String::as_str),
            Some("authorizing")
        );
    }

    #[test]
    fn treats_only_a_fully_booted_device_as_healthy() {
        assert_eq!(
            AdbDeviceState::Online.facts().health,
            ConnectionHealth::Ready
        );

        for state in [
            AdbDeviceState::Offline,
            AdbDeviceState::Unauthorized,
            AdbDeviceState::NoPermissions,
        ] {
            assert_eq!(state.facts().health, ConnectionHealth::Blocked);
        }
    }

    #[test]
    fn gives_every_faulted_state_a_diagnostic_code() {
        for state in [
            AdbDeviceState::Offline,
            AdbDeviceState::Unauthorized,
            AdbDeviceState::NoPermissions,
            AdbDeviceState::Unknown("authorizing".to_string()),
        ] {
            assert_ne!(state.facts().diagnostic, DiagnosticCode::None);
        }
    }

    #[test]
    fn points_transient_states_at_the_phone_rather_than_at_adb() {
        let diagnostic = |state: &str| {
            AdbDeviceState::Unknown(state.to_string())
                .facts()
                .diagnostic
        };

        assert_eq!(diagnostic("authorizing"), DiagnosticCode::UsbUnauthorized);
        assert_eq!(diagnostic("connecting"), DiagnosticCode::DeviceOffline);
        assert_eq!(diagnostic("rescue"), DiagnosticCode::AdbConflict);
    }

    #[test]
    fn does_not_flag_flashing_states_as_faults() {
        for state in [
            AdbDeviceState::Recovery,
            AdbDeviceState::Sideload,
            AdbDeviceState::Bootloader,
        ] {
            assert_eq!(state.facts().diagnostic, DiagnosticCode::None);
        }
    }
}
