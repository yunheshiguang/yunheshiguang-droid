use crate::workspace::contract::DiagnosticCode;
use crate::workspace::tool_archive::ArchiveError;

#[derive(Clone, Debug)]
pub struct AndroidPlatformToolsInstallFailure {
    pub code: DiagnosticCode,
    pub message: String,
}

impl AndroidPlatformToolsInstallFailure {
    fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(super) fn download(error: impl ToString) -> Self {
        Self::new(
            DiagnosticCode::AndroidPlatformToolsDownloadFailed,
            error.to_string(),
        )
    }

    pub(super) fn archive(error: impl ToString) -> Self {
        Self::new(
            DiagnosticCode::AndroidPlatformToolsArchiveInvalid,
            error.to_string(),
        )
    }

    pub(in crate::workspace::android_tools) fn install(error: impl ToString) -> Self {
        Self::new(
            DiagnosticCode::AndroidPlatformToolsInstallFailed,
            error.to_string(),
        )
    }

    pub(super) fn integrity(error: impl ToString) -> Self {
        Self::new(
            DiagnosticCode::AndroidPlatformToolsIntegrityFailed,
            error.to_string(),
        )
    }

    pub(super) fn unsupported_host(error: impl ToString) -> Self {
        Self::new(
            DiagnosticCode::AndroidPlatformToolsHostUnsupported,
            error.to_string(),
        )
    }
}

impl From<ArchiveError> for AndroidPlatformToolsInstallFailure {
    fn from(error: ArchiveError) -> Self {
        match error {
            ArchiveError::Download(message) => Self::download(message),
            ArchiveError::Integrity(message) => Self::integrity(message),
            ArchiveError::Archive(message) => Self::archive(message),
        }
    }
}
