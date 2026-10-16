use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

use url::Host;

use crate::contract::yunheshiguang::connection::v1::{ConnectionResult, NetworkEndpoint};

pub trait ConnectionProvider: Send + Sync {
    fn connect(&self, endpoint: &Endpoint) -> ConnectionResult;

    fn pair(&self, endpoint: &Endpoint, code: &PairingCode) -> ConnectionResult;

    fn disconnect(&self, device: &NetworkSerial) -> ConnectionResult;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EndpointError {
    InvalidHost,
    InvalidPort,
    InvalidPairingCode,
    InvalidDevice,
}

const MAX_HOST_LEN: usize = 253;

const PAIRING_CODE_LEN: usize = 6;

fn normalize_host(raw: &str) -> Option<String> {
    let host = raw.trim();
    let host = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(host);

    if host.is_empty() || host.len() > MAX_HOST_LEN || host.starts_with('-') {
        return None;
    }

    if let Ok(address) = host.parse::<Ipv6Addr>() {
        return Some(address.to_string());
    }

    match Host::parse(host) {
        Ok(Host::Ipv4(_)) => host
            .parse::<Ipv4Addr>()
            .ok()
            .map(|address| address.to_string()),
        Ok(Host::Domain(domain)) => Some(domain),
        _ => None,
    }
}

fn parse_port(port: u32) -> Option<u16> {
    u16::try_from(port).ok().filter(|port| *port != 0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    host: String,
    port: u16,
}

impl Endpoint {
    pub fn parse(host: &str, port: u32) -> Result<Self, EndpointError> {
        let host = normalize_host(host).ok_or(EndpointError::InvalidHost)?;
        let port = parse_port(port).ok_or(EndpointError::InvalidPort)?;

        Ok(Self { host, port })
    }

    pub fn from_contract(endpoint: Option<&NetworkEndpoint>) -> Result<Self, EndpointError> {
        let endpoint = endpoint.cloned().unwrap_or_default();

        Self::parse(&endpoint.host, endpoint.port)
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(formatter, "[{}]:{}", self.host, self.port)
        } else {
            write!(formatter, "{}:{}", self.host, self.port)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingCode(String);

impl PairingCode {
    pub fn parse(raw: &str) -> Result<Self, EndpointError> {
        let code = raw.trim();

        if code.len() == PAIRING_CODE_LEN && code.bytes().all(|byte| byte.is_ascii_digit()) {
            Ok(Self(code.to_string()))
        } else {
            Err(EndpointError::InvalidPairingCode)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub const ADB_TLS_CONNECT_SERVICE: &str = "._adb-tls-connect._tcp";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkSerial(String);

impl NetworkSerial {
    pub fn parse(raw: &str) -> Result<Self, EndpointError> {
        let serial = raw.trim();
        let host_port = serial.rsplit_once(':').is_some_and(|(host, port)| {
            port.parse::<u32>().ok().and_then(parse_port).is_some()
                && normalize_host(host).is_some()
        });
        let discovered = serial
            .strip_suffix(ADB_TLS_CONNECT_SERVICE)
            .is_some_and(|instance| {
                !instance.is_empty() && !instance.chars().any(char::is_control)
            });
        let valid = host_port || discovered;

        if valid {
            Ok(Self(serial.to_string()))
        } else {
            Err(EndpointError::InvalidDevice)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Endpoint, EndpointError, NetworkSerial, PairingCode};

    #[test]
    fn accepts_ip_literals_and_host_names() {
        for host in [
            "192.168.1.20",
            " 10.0.0.2 ",
            "::1",
            "[fe80::1]",
            "pixel-7.local",
        ] {
            assert!(
                Endpoint::parse(host, 5555).is_ok(),
                "{host} should be accepted"
            );
        }

        assert_eq!(
            Endpoint::parse("[::1]", 37000).unwrap().to_string(),
            "[::1]:37000"
        );
    }

    #[test]
    fn hands_out_the_canonical_host() {
        let canonical = |host: &str| Endpoint::parse(host, 5555).unwrap().to_string();

        assert_eq!(canonical("Pixel-7.LOCAL"), "pixel-7.local:5555");
        assert_eq!(canonical("例子.local"), "xn--fsqu00a.local:5555");
        assert_eq!(canonical("fe80:0:0:0:0:0:0:1"), "[fe80::1]:5555");
    }

    #[test]
    fn rejects_hosts_adb_would_misread() {
        for host in [
            "",
            "  ",
            "-s",
            "a b",
            "host/path",
            "user@host",
            "192.168.1.300",
            "192.168.001.020",
            "010.0.0.1",
            "0x7f.0.0.1",
            "192.168.1",
        ] {
            assert_eq!(
                Endpoint::parse(host, 5555).unwrap_err(),
                EndpointError::InvalidHost,
                "{host:?}"
            );
        }

        for port in [0, 65_536] {
            assert_eq!(
                Endpoint::parse("192.168.1.20", port).unwrap_err(),
                EndpointError::InvalidPort
            );
        }
    }

    #[test]
    fn accepts_only_six_digit_pairing_codes() {
        assert_eq!(PairingCode::parse(" 123456 ").unwrap().as_str(), "123456");

        for code in ["12345", "1234567", "abcdef", ""] {
            assert_eq!(
                PairingCode::parse(code).unwrap_err(),
                EndpointError::InvalidPairingCode
            );
        }
    }

    #[test]
    fn only_accepts_network_device_ids() {
        for id in [
            "192.168.1.20:5555",
            "[::1]:5555",
            "pixel-7.local:41234",
            "adb-R58M20ABCDE-Qz1aBc._adb-tls-connect._tcp",
        ] {
            assert_eq!(NetworkSerial::parse(id).unwrap().as_str(), id);
        }

        for id in [
            "",
            "   ",
            "R58M20ABCDE",
            "emulator-5554",
            ":5555",
            "host:0",
            "host:x",
            "._adb-tls-connect._tcp",
        ] {
            assert_eq!(
                NetworkSerial::parse(id).unwrap_err(),
                EndpointError::InvalidDevice,
                "{id:?}"
            );
        }
    }
}
