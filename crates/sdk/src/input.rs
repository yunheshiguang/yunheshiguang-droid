pub use crate::contract::yunheshiguang::input::v1::{InputChannelInfo, InputKind};
use crate::contract::yunheshiguang::workspace::v1::UiText;
use crate::device::DeviceShell;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputRequest<'a> {
    Text(&'a str),
    Clipboard(&'a str),
}

impl InputRequest<'_> {
    pub fn kind(&self) -> InputKind {
        match self {
            Self::Text(_) => InputKind::Text,
            Self::Clipboard(_) => InputKind::Clipboard,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Text(text) | Self::Clipboard(text) => text,
        }
    }
}

pub trait InputChannel: Send + Sync {
    fn describe(&self) -> InputChannelInfo;

    fn accepts(&self, device: &dyn DeviceShell, request: &InputRequest<'_>) -> bool {
        let _ = device;
        self.describe().kinds.contains(&request.kind().into())
    }

    fn deliver(&self, device: &dyn DeviceShell, request: &InputRequest<'_>) -> Result<(), UiText>;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::device::ShellOutput;
    use crate::vendor::DeviceIdentity;

    struct NoShell;

    impl DeviceShell for NoShell {
        fn identity(&self) -> Option<&DeviceIdentity> {
            None
        }

        fn shell(&self, _args: &[&str]) -> Result<ShellOutput, UiText> {
            Ok(ShellOutput::default())
        }

        fn push(&self, _local: &Path, _remote: &str) -> Result<(), UiText> {
            Ok(())
        }
    }

    struct TextOnly;

    impl InputChannel for TextOnly {
        fn describe(&self) -> InputChannelInfo {
            InputChannelInfo {
                id: "test.text-only".to_string(),
                kinds: vec![InputKind::Text.into()],
                ..InputChannelInfo::default()
            }
        }

        fn deliver(
            &self,
            _device: &dyn DeviceShell,
            _request: &InputRequest<'_>,
        ) -> Result<(), UiText> {
            Ok(())
        }
    }

    #[test]
    fn takes_only_the_kinds_a_channel_declares_by_default() {
        assert!(TextOnly.accepts(&NoShell, &InputRequest::Text("你好")));
        assert!(!TextOnly.accepts(&NoShell, &InputRequest::Clipboard("你好")));
    }

    #[test]
    fn exposes_the_text_and_kind_of_a_request() {
        let request = InputRequest::Clipboard("line 1\nline 2");

        assert_eq!(request.kind(), InputKind::Clipboard);
        assert_eq!(request.text(), "line 1\nline 2");
    }
}
