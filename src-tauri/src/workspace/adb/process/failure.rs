use crate::workspace::adb::error::AdbErrorKind;

pub fn output_error_message(stderr: &[u8], fallback: String) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_string();

    if message.is_empty() {
        format!("adb exited with {fallback}")
    } else {
        message
    }
}

pub fn classify_exit_error(message: &str) -> AdbErrorKind {
    let message = message.to_ascii_lowercase();

    if message.contains("permission denied") || message.contains("access is denied") {
        AdbErrorKind::PermissionDenied
    } else {
        AdbErrorKind::NonZeroExit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_permission_failures() {
        assert_eq!(
            classify_exit_error("remote couldn't create file: Permission denied"),
            AdbErrorKind::PermissionDenied
        );
        assert_eq!(
            classify_exit_error("device offline"),
            AdbErrorKind::NonZeroExit
        );
    }

    #[test]
    fn falls_back_to_the_exit_status_when_stderr_is_silent() {
        assert_eq!(
            output_error_message(b"   ", "exit code: 1".to_string()),
            "adb exited with exit code: 1"
        );
    }
}
