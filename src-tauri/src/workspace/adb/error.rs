use std::fmt;
use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdbErrorKind {
    NotFound,
    PermissionDenied,
    Timeout,
    OutputLimitExceeded,
    NonZeroExit,
    InvalidOutput,
    Io,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdbCommandError {
    pub kind: AdbErrorKind,
    pub message: String,
    pub exit_code: Option<i32>,
}

impl AdbCommandError {
    pub fn new(kind: AdbErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            exit_code: None,
        }
    }

    pub fn from_io(error: io::Error) -> Self {
        let kind = match error.kind() {
            io::ErrorKind::NotFound => AdbErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => AdbErrorKind::PermissionDenied,
            _ => AdbErrorKind::Io,
        };

        Self::new(kind, error.to_string())
    }

    pub fn non_zero(exit_code: Option<i32>, message: impl Into<String>) -> Self {
        Self {
            kind: AdbErrorKind::NonZeroExit,
            message: message.into(),
            exit_code,
        }
    }

    pub fn exited(kind: AdbErrorKind, exit_code: Option<i32>, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            exit_code,
        }
    }
}

impl From<String> for AdbCommandError {
    fn from(message: String) -> Self {
        Self::new(AdbErrorKind::InvalidOutput, message)
    }
}

impl fmt::Display for AdbCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AdbCommandError {}
