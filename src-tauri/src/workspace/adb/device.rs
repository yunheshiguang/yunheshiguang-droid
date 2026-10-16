#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdbDeviceState {
    Online,
    Offline,
    Unauthorized,
    NoPermissions,
    Recovery,
    Sideload,
    Bootloader,
    Unknown(String),
}

impl AdbDeviceState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Online => "device",
            Self::Offline => "offline",
            Self::Unauthorized => "unauthorized",
            Self::NoPermissions => "no permissions",
            Self::Recovery => "recovery",
            Self::Sideload => "sideload",
            Self::Bootloader => "bootloader",
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdbDevice {
    pub serial: String,
    pub model: Option<String>,
    pub device: Option<String>,
    pub product: Option<String>,
    pub usb: Option<String>,
    pub transport_id: Option<String>,
    pub state: AdbDeviceState,
}

pub(crate) fn parse_device_line(line: &str) -> Option<AdbDevice> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("List of devices")
        || trimmed.starts_with('*')
        || trimmed.starts_with("adb:")
    {
        return None;
    }

    let (serial, remainder) = split_once_whitespace(trimmed)?;
    let (state, metadata) = parse_state(remainder);
    let mut model = None;
    let mut device = None;
    let mut product = None;
    let mut usb = None;
    let mut transport_id = None;

    for item in metadata.split_whitespace() {
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };

        match key {
            "model" => model = Some(value.replace('_', " ")),
            "device" => device = Some(value.to_string()),
            "product" => product = Some(value.to_string()),
            "usb" => usb = Some(value.to_string()),
            "transport_id" => transport_id = Some(value.to_string()),
            _ => {}
        }
    }

    Some(AdbDevice {
        serial: serial.to_string(),
        model,
        device,
        product,
        usb,
        transport_id,
        state,
    })
}

pub(crate) fn parse_device_list(stdout: &[u8]) -> Result<Vec<AdbDevice>, String> {
    let text = std::str::from_utf8(stdout).map_err(|_| "adb device output is not valid UTF-8")?;
    let mut found_header = false;
    let mut devices = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("List of devices attached") {
            found_header = true;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('*') || trimmed.starts_with("adb:") {
            continue;
        }
        if !found_header {
            return Err("adb device output contains data before its header".to_string());
        }
        let device =
            parse_device_line(line).ok_or_else(|| format!("invalid adb device line: {trimmed}"))?;
        devices.push(device);
    }

    if !found_header {
        return Err("adb device output is missing its header".to_string());
    }
    Ok(devices)
}

fn split_once_whitespace(value: &str) -> Option<(&str, &str)> {
    let index = value.find(char::is_whitespace)?;
    let (first, remainder) = value.split_at(index);
    Some((first, remainder.trim_start()))
}

fn parse_state(remainder: &str) -> (AdbDeviceState, &str) {
    if let Some(metadata) = remainder.strip_prefix("no permissions") {
        return (AdbDeviceState::NoPermissions, metadata.trim_start());
    }

    let (state, metadata) = split_once_whitespace(remainder).unwrap_or((remainder, ""));
    let state = match state {
        "device" => AdbDeviceState::Online,
        "offline" => AdbDeviceState::Offline,
        "unauthorized" => AdbDeviceState::Unauthorized,
        "recovery" => AdbDeviceState::Recovery,
        "sideload" => AdbDeviceState::Sideload,
        "bootloader" => AdbDeviceState::Bootloader,
        other => AdbDeviceState::Unknown(other.to_string()),
    };
    (state, metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_long_device_metadata() {
        let device = parse_device_line(
            "012345 device usb:1-2 product:foo model:Pixel_8 device:shiba transport_id:2",
        )
        .expect("device should parse");

        assert_eq!(device.serial, "012345");
        assert_eq!(device.state, AdbDeviceState::Online);
        assert_eq!(device.model.as_deref(), Some("Pixel 8"));
        assert_eq!(device.product.as_deref(), Some("foo"));
        assert_eq!(device.usb.as_deref(), Some("1-2"));
        assert_eq!(device.transport_id.as_deref(), Some("2"));
    }

    #[test]
    fn parses_linux_no_permissions_state() {
        let device = parse_device_line(
            "???????????? no permissions (user in plugdev group; are your udev rules wrong?)",
        )
        .expect("device should parse");

        assert_eq!(device.state, AdbDeviceState::NoPermissions);
    }

    #[test]
    fn accepts_crlf_and_ignores_protocol_lines() {
        let output = b"List of devices attached\r\nemulator-5554\tdevice product:sdk\r\n\r\n";
        let devices = parse_device_list(output).expect("list should parse");

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "emulator-5554");
    }

    #[test]
    fn distinguishes_empty_lists_from_invalid_output() {
        assert_eq!(
            parse_device_list(b"List of devices attached\n")
                .expect("an empty device list is valid"),
            Vec::new()
        );
        assert!(parse_device_list(b"unexpected output\n").is_err());
        assert!(parse_device_list(b"List of devices attached\nmalformed-line\n").is_err());
        assert!(parse_device_list(b"List of devices attached\n\xff\n").is_err());
    }
}
