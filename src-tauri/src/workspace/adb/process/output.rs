use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use crate::workspace::adb::error::{AdbCommandError, AdbErrorKind};

const CLEANUP_TIMEOUT: Duration = Duration::from_secs(1);

pub enum StdoutTarget {
    Capture,
    File(File),
}

#[derive(Debug)]
pub struct BoundedOutput {
    pub bytes: Vec<u8>,
    pub exceeded: bool,
}

pub type OutputReader = Receiver<io::Result<BoundedOutput>>;

pub fn read_bounded(
    mut reader: impl Read,
    limit: usize,
    mut target: StdoutTarget,
) -> io::Result<BoundedOutput> {
    let mut bytes = Vec::with_capacity(limit.min(16 * 1024));
    let mut buffer = [0_u8; 8 * 1024];
    let mut exceeded = false;
    let mut retained_bytes = 0_usize;

    loop {
        let read = reader.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        let remaining = limit.saturating_sub(retained_bytes);
        let retained = remaining.min(read);

        match &mut target {
            StdoutTarget::Capture => bytes.extend_from_slice(&buffer[..retained]),
            StdoutTarget::File(file) => file.write_all(&buffer[..retained])?,
        }

        retained_bytes += retained;
        exceeded |= retained < read;
    }

    if let StdoutTarget::File(file) = &mut target {
        file.flush()?;
    }

    Ok(BoundedOutput { bytes, exceeded })
}

pub fn spawn_reader(
    reader: impl Read + Send + 'static,
    limit: usize,
    target: StdoutTarget,
) -> OutputReader {
    let (sender, receiver) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let _ = sender.send(read_bounded(reader, limit, target));
    });

    receiver
}

pub fn receive_reader(
    reader: OutputReader,
    deadline: Instant,
) -> Result<BoundedOutput, AdbCommandError> {
    let remaining = deadline.saturating_duration_since(Instant::now());

    match reader.recv_timeout(remaining) {
        Ok(result) => result.map_err(AdbCommandError::from_io),
        Err(RecvTimeoutError::Timeout) => Err(AdbCommandError::new(
            AdbErrorKind::Timeout,
            "command output pipes did not close before the timeout",
        )),
        Err(RecvTimeoutError::Disconnected) => Err(AdbCommandError::new(
            AdbErrorKind::Io,
            "output reader stopped unexpectedly",
        )),
    }
}

pub fn finish_readers(stdout: OutputReader, stderr: OutputReader) {
    let _ = stdout.recv_timeout(CLEANUP_TIMEOUT);
    let _ = stderr.recv_timeout(CLEANUP_TIMEOUT);
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek, SeekFrom};

    use tempfile::tempfile;

    use super::*;

    #[test]
    fn drains_but_caps_process_output() {
        let input = vec![b'x'; 32];
        let output =
            read_bounded(input.as_slice(), 8, StdoutTarget::Capture).expect("read should succeed");

        assert_eq!(output.bytes, vec![b'x'; 8]);
        assert!(output.exceeded);
    }

    #[test]
    fn streams_only_the_bounded_prefix_to_a_file() {
        let input = vec![b'x'; 32];
        let mut file = tempfile().expect("temporary file should exist");
        let writer = file.try_clone().expect("file should clone");

        let output = read_bounded(input.as_slice(), 8, StdoutTarget::File(writer))
            .expect("read should work");

        file.seek(SeekFrom::Start(0)).expect("file should seek");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).expect("file should read");
        assert!(output.bytes.is_empty());
        assert!(output.exceeded);
        assert_eq!(bytes, vec![b'x'; 8]);
    }
}
