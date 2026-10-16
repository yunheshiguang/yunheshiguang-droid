use yunheshiguang_sdk::connection::ADB_TLS_CONNECT_SERVICE;

use crate::workspace::contract::{AdbTransportKind, ConnectionType};

use super::device::AdbDevice;

const EMULATOR_SERIAL_PREFIX: &str = "emulator-";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Usb,
    Network,
    Emulator,
    Unknown,
}

pub fn classify(device: &AdbDevice) -> Transport {
    let serial = device.serial.as_str();

    if serial.starts_with(EMULATOR_SERIAL_PREFIX) {
        Transport::Emulator
    } else if device.usb.is_some() {
        Transport::Usb
    } else if serial.contains(':') || serial.ends_with(ADB_TLS_CONNECT_SERVICE) {
        Transport::Network
    } else if serial.trim().is_empty() {
        Transport::Unknown
    } else {
        Transport::Usb
    }
}

impl Transport {
    pub fn connection_type(self) -> ConnectionType {
        match self {
            Self::Usb => ConnectionType::Usb,
            Self::Network | Self::Emulator => ConnectionType::Wifi,
            Self::Unknown => ConnectionType::Unspecified,
        }
    }

    pub fn kind(self) -> AdbTransportKind {
        match self {
            Self::Usb => AdbTransportKind::Usb,
            Self::Network => AdbTransportKind::Wifi,
            Self::Emulator => AdbTransportKind::Emulator,
            Self::Unknown => AdbTransportKind::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::adb::device::AdbDeviceState;

    fn device(serial: &str, usb: Option<&str>) -> AdbDevice {
        AdbDevice {
            serial: serial.to_string(),
            model: None,
            device: None,
            product: None,
            usb: usb.map(str::to_string),
            transport_id: None,
            state: AdbDeviceState::Online,
        }
    }

    #[test]
    fn recognizes_an_emulator_before_reading_the_serial_shape() {
        assert_eq!(
            classify(&device("emulator-5554", None)),
            Transport::Emulator
        );
    }

    #[test]
    fn keeps_both_contract_enums_consistent_for_an_emulator() {
        let transport = classify(&device("emulator-5554", None));

        assert_eq!(transport.kind(), AdbTransportKind::Emulator);
        assert_ne!(transport.connection_type(), ConnectionType::Usb);
    }

    #[test]
    fn trusts_the_usb_field_over_the_serial_shape() {
        assert_eq!(
            classify(&device("R58M20ABCDE", Some("1-2.3"))),
            Transport::Usb
        );
    }

    #[test]
    fn reads_a_host_port_serial_as_a_network_transport() {
        assert_eq!(
            classify(&device("192.168.1.30:5555", None)),
            Transport::Network
        );
        assert_eq!(
            classify(&device("192.168.1.30:5555", None)).connection_type(),
            ConnectionType::Wifi
        );
    }

    #[test]
    fn reads_a_plain_serial_as_usb_even_without_the_usb_field() {
        assert_eq!(classify(&device("R58M20ABCDE", None)), Transport::Usb);
    }

    #[test]
    fn reads_a_wireless_debugging_service_name_as_a_network_transport() {
        assert_eq!(
            classify(&device(
                "adb-R58M20ABCDE-Qz1aBc._adb-tls-connect._tcp",
                None
            )),
            Transport::Network
        );
    }

    #[test]
    fn leaves_an_empty_serial_unspecified() {
        let transport = classify(&device("", None));

        assert_eq!(transport, Transport::Unknown);
        assert_eq!(transport.connection_type(), ConnectionType::Unspecified);
        assert_eq!(transport.kind(), AdbTransportKind::Other);
    }
}
