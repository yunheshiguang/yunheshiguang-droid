pub use crate::contract::yunheshiguang::scripts::v1::{ScriptEngineInfo, ScriptLanguage};
use crate::contract::yunheshiguang::workspace::v1::UiText;
use crate::device::DeviceShell;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptRequest<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub language: ScriptLanguage,
    pub body: &'a str,
}

impl<'a> ScriptRequest<'a> {
    pub fn new(id: &'a str, name: &'a str, language: ScriptLanguage, body: &'a str) -> Self {
        Self {
            id,
            name,
            language,
            body,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptOutcome {
    pub exit_code: Option<i32>,
    pub output: String,
    pub truncated: bool,
}

impl ScriptOutcome {
    pub fn finished(exit_code: i32, output: impl Into<String>) -> Self {
        Self {
            exit_code: Some(exit_code),
            output: output.into(),
            truncated: false,
        }
    }

    pub fn launched() -> Self {
        Self::default()
    }

    pub fn with_truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }
}

pub trait ScriptEngine: Send + Sync {
    fn describe(&self) -> ScriptEngineInfo;

    fn accepts(&self, device: &dyn DeviceShell, script: &ScriptRequest<'_>) -> bool {
        let _ = device;
        self.describe().languages.contains(&script.language.into())
    }

    fn run(
        &self,
        device: &dyn DeviceShell,
        script: &ScriptRequest<'_>,
    ) -> Result<ScriptOutcome, UiText>;
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

    struct ShellOnly;

    impl ScriptEngine for ShellOnly {
        fn describe(&self) -> ScriptEngineInfo {
            ScriptEngineInfo {
                id: "test.shell".to_string(),
                languages: vec![ScriptLanguage::Shell.into()],
                captures_output: true,
                ..ScriptEngineInfo::default()
            }
        }

        fn run(
            &self,
            _device: &dyn DeviceShell,
            _script: &ScriptRequest<'_>,
        ) -> Result<ScriptOutcome, UiText> {
            Ok(ScriptOutcome::finished(0, "ok"))
        }
    }

    #[test]
    fn takes_only_the_languages_an_engine_declares_by_default() {
        let shell = ScriptRequest::new("a", "a", ScriptLanguage::Shell, "echo hi");
        let javascript = ScriptRequest::new("b", "b", ScriptLanguage::Javascript, "toast('hi')");

        assert!(ShellOnly.accepts(&NoShell, &shell));
        assert!(!ShellOnly.accepts(&NoShell, &javascript));
    }

    #[test]
    fn tells_a_finished_script_from_one_that_was_only_launched() {
        assert_eq!(ScriptOutcome::finished(3, "boom").exit_code, Some(3));
        assert_eq!(ScriptOutcome::launched().exit_code, None);
        assert!(ScriptOutcome::launched().with_truncated(true).truncated);
    }

    #[test]
    fn falls_back_to_the_plain_shell_when_a_device_has_no_longer_timeout() {
        let output = NoShell
            .shell_with_timeout(&["true"], std::time::Duration::from_secs(600))
            .unwrap();

        assert_eq!(output, ShellOutput::default());
    }
}
