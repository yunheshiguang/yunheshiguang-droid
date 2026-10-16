use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::{App, Runtime};

use crate::workspace::contract::DevicesChanged;
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

pub const DEVICES_CHANGED_EVENT: &str = "workspace:devices-changed";

const DEFAULT_SERVER_PORT: u16 = 5037;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const FIRST_RETRY: Duration = Duration::from_secs(1);
const LAST_RETRY: Duration = Duration::from_secs(30);
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

fn server_address() -> SocketAddr {
    let port = std::env::var("ANDROID_ADB_SERVER_PORT")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(DEFAULT_SERVER_PORT);

    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

fn read_message(stream: &mut impl Read) -> io::Result<String> {
    let mut header = [0; 4];
    stream.read_exact(&mut header)?;
    let length = std::str::from_utf8(&header)
        .ok()
        .and_then(|hex| usize::from_str_radix(hex, 16).ok())
        .filter(|length| *length <= MAX_MESSAGE_BYTES)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid adb message length"))?;
    let mut body = vec![0; length];
    stream.read_exact(&mut body)?;

    Ok(String::from_utf8_lossy(&body).into_owned())
}

fn open_service(stream: &mut (impl Read + Write), service: &str) -> io::Result<()> {
    write!(stream, "{:04x}{service}", service.len())?;
    stream.flush()?;

    let mut status = [0; 4];
    stream.read_exact(&mut status)?;
    match &status {
        b"OKAY" => Ok(()),
        b"FAIL" => Err(io::Error::other(read_message(stream)?)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected adb server status",
        )),
    }
}

fn track(
    stream: &mut (impl Read + Write),
    last: &mut Option<String>,
    mut changed: impl FnMut(),
) -> io::Result<()> {
    open_service(stream, "host:track-devices")?;

    loop {
        let devices = read_message(stream)?;
        if last.as_deref() != Some(devices.as_str()) {
            *last = Some(devices);
            changed();
        }
    }
}

fn watch(events: Arc<dyn WorkspaceEvents>) {
    let address = server_address();
    let mut last = None;
    let mut retry = FIRST_RETRY;

    loop {
        let tracked =
            TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).and_then(|mut stream| {
                log::debug!("adb watch: tracking devices");
                retry = FIRST_RETRY;
                track(&mut stream, &mut last, || {
                    notify(
                        events.as_ref(),
                        DEVICES_CHANGED_EVENT,
                        &DevicesChanged {
                            ..Default::default()
                        },
                    );
                })
            });

        if let Err(error) = tracked {
            log::debug!("adb watch: {:?}, retrying in {retry:?}", error.kind());
        }
        thread::sleep(retry);
        retry = (retry * 2).min(LAST_RETRY);
    }
}

pub struct DeviceWatchModule;

impl<R: Runtime> WorkspaceModule<R> for DeviceWatchModule {
    fn id(&self) -> &'static str {
        "device-watch"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let events = dependency::<dyn WorkspaceEvents, R>(app)?;

        thread::Builder::new()
            .name("adb-device-watch".to_string())
            .spawn(move || watch(events))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    struct Server {
        replies: Cursor<Vec<u8>>,
        sent: Vec<u8>,
    }

    impl Read for Server {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.replies.read(buffer)
        }
    }

    impl Write for Server {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.sent.write(buffer)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn server(replies: &str) -> Server {
        Server {
            replies: Cursor::new(replies.as_bytes().to_vec()),
            sent: Vec::new(),
        }
    }

    #[test]
    fn reports_each_new_device_list_once_until_the_server_hangs_up() {
        let mut adb = server("OKAY0000000eabc123\tdevice\n000eabc123\tdevice\n0000");
        let mut last = None;
        let mut changes = 0;

        let ended = track(&mut adb, &mut last, || changes += 1);

        assert_eq!(adb.sent, b"0012host:track-devices");
        assert_eq!(ended.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(changes, 3, "empty, one device, empty again");
        assert_eq!(last.as_deref(), Some(""));
    }

    #[test]
    fn keeps_the_last_list_across_reconnects() {
        let mut last = Some("abc123\tdevice\n".to_string());
        let mut changes = 0;

        let _ = track(&mut server("OKAY000eabc123\tdevice\n"), &mut last, || {
            changes += 1;
        });

        assert_eq!(changes, 0);
    }

    #[test]
    fn explains_a_refused_service() {
        let refused = open_service(&mut server("FAIL0007unknown"), "host:track-devices");

        assert_eq!(refused.unwrap_err().to_string(), "unknown");
        assert!(read_message(&mut Cursor::new(b"zzzz".to_vec())).is_err());
    }
}
