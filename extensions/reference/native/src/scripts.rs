use std::io::Write;

use tempfile::NamedTempFile;
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::UiText;
use yunheshiguang_sdk::device::DeviceShell;
use yunheshiguang_sdk::scripts::{
    ScriptEngine, ScriptEngineInfo, ScriptLanguage, ScriptOutcome, ScriptRequest,
};
use yunheshiguang_sdk::text::{ui_text, ui_text_with_params};

use crate::messages::*;

pub const AUTOJS6_ENGINE_ID: &str = "reference.autojs6";
const PACKAGE: &str = "org.autojs.autojs6";
const RUN_ACTIVITY: &str = "org.autojs.autojs6/org.autojs.autojs.external.open.RunIntentActivity";
const REMOTE_DIRECTORY: &str = "/sdcard/Download/yunheshiguang-scripts";

fn remote_path(id: &str) -> String {
    let name = id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(64)
        .collect::<String>();

    format!(
        "{REMOTE_DIRECTORY}/{}.js",
        if name.is_empty() { "script" } else { &name }
    )
}

pub struct AutoJs6Engine;

impl AutoJs6Engine {
    fn failed(message: impl Into<String>) -> UiText {
        ui_text_with_params(
            REFERENCE_SCRIPTS_FAILED_KEY,
            "",
            [("message", message.into())],
        )
    }
}

impl ScriptEngine for AutoJs6Engine {
    fn describe(&self) -> ScriptEngineInfo {
        ScriptEngineInfo {
            id: AUTOJS6_ENGINE_ID.to_string(),
            label: ui_text(REFERENCE_SCRIPTS_LABEL_KEY).into(),
            description: ui_text(REFERENCE_SCRIPTS_DESCRIPTION_KEY).into(),
            languages: vec![ScriptLanguage::Javascript.into()],
            captures_output: false,
            setup: vec![
                ui_text(REFERENCE_SCRIPTS_SETUP_INSTALL_KEY),
                ui_text(REFERENCE_SCRIPTS_SETUP_STORAGE_KEY),
            ],
            ..Default::default()
        }
    }

    fn accepts(&self, device: &dyn DeviceShell, script: &ScriptRequest<'_>) -> bool {
        script.language == ScriptLanguage::Javascript
            && device
                .shell(&["pm", "path", PACKAGE])
                .is_ok_and(|output| output.stdout.contains("package:"))
    }

    fn run(
        &self,
        device: &dyn DeviceShell,
        script: &ScriptRequest<'_>,
    ) -> Result<ScriptOutcome, UiText> {
        let mut local = NamedTempFile::new().map_err(|error| Self::failed(error.to_string()))?;
        local
            .write_all(script.body.as_bytes())
            .map_err(|error| Self::failed(error.to_string()))?;
        let remote = remote_path(script.id);

        device.shell(&["mkdir", "-p", REMOTE_DIRECTORY])?;
        device.push(local.path(), &remote)?;
        let output = device.shell(&[
            "am",
            "start",
            "-W",
            "-n",
            RUN_ACTIVITY,
            "-d",
            &format!("file://{remote}"),
            "-t",
            "text/javascript",
        ])?;

        if !output
            .stdout
            .lines()
            .any(|line| line.trim() == "Status: ok")
        {
            return Err(Self::failed(output.stdout.trim()));
        }
        Ok(ScriptOutcome::launched())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;

    use yunheshiguang_sdk::device::ShellOutput;
    use yunheshiguang_sdk::vendor::DeviceIdentity;

    use super::*;

    struct Device {
        installed: bool,
        started: &'static str,
        calls: RefCell<Vec<Vec<String>>>,
        pushed: RefCell<Vec<(String, String)>>,
    }

    impl Device {
        fn new(installed: bool) -> Self {
            Self {
                installed,
                started: "Starting: Intent { dat=file:///sdcard/Download/x.js typ=text/javascript }\nStatus: ok\nLaunchState: COLD\nComplete",
                calls: RefCell::new(Vec::new()),
                pushed: RefCell::new(Vec::new()),
            }
        }
    }

    impl DeviceShell for Device {
        fn identity(&self) -> Option<&DeviceIdentity> {
            None
        }

        fn shell(&self, args: &[&str]) -> Result<ShellOutput, UiText> {
            self.calls
                .borrow_mut()
                .push(args.iter().map(ToString::to_string).collect());

            Ok(ShellOutput::new(match args[0] {
                "pm" if self.installed => "package:/data/app/org.autojs.autojs6/base.apk",
                "am" => self.started,
                _ => "",
            }))
        }

        fn push(&self, local: &Path, remote: &str) -> Result<(), UiText> {
            let body = fs::read_to_string(local).unwrap();
            self.pushed.borrow_mut().push((remote.to_string(), body));
            Ok(())
        }
    }

    fn script(language: ScriptLanguage) -> ScriptRequest<'static> {
        ScriptRequest::new("daily-check", "Daily", language, "toast('hi')")
    }

    #[test]
    fn takes_javascript_only_on_devices_with_autojs6() {
        assert!(AutoJs6Engine.accepts(&Device::new(true), &script(ScriptLanguage::Javascript)));
        assert!(!AutoJs6Engine.accepts(&Device::new(false), &script(ScriptLanguage::Javascript)));
        assert!(!AutoJs6Engine.accepts(&Device::new(true), &script(ScriptLanguage::Shell)));
    }

    #[test]
    fn pushes_the_script_and_asks_autojs6_to_run_it() {
        let device = Device::new(true);

        assert_eq!(
            AutoJs6Engine.run(&device, &script(ScriptLanguage::Javascript)),
            Ok(ScriptOutcome::launched())
        );
        let pushed = device.pushed.borrow();
        assert_eq!(
            pushed[0],
            (
                "/sdcard/Download/yunheshiguang-scripts/daily-check.js".to_string(),
                "toast('hi')".to_string()
            )
        );
        let start = device.calls.borrow().last().cloned().unwrap();
        assert_eq!(start[..5], ["am", "start", "-W", "-n", RUN_ACTIVITY]);
        assert_eq!(
            start[6],
            "file:///sdcard/Download/yunheshiguang-scripts/daily-check.js"
        );
    }

    #[test]
    fn reports_what_the_activity_manager_said_when_it_did_not_start() {
        let device = Device {
            started: "Starting: Intent { dat=file:///sdcard/Download/x.js }\nError type 3\nError: Activity class does not exist.",
            ..Device::new(true)
        };

        let failure = AutoJs6Engine
            .run(&device, &script(ScriptLanguage::Javascript))
            .unwrap_err();

        assert_eq!(failure.key, REFERENCE_SCRIPTS_FAILED_KEY);
        assert!(failure.params["message"].contains("does not exist"));
    }
}
