use std::io::Write;
use std::time::Duration;

use tempfile::NamedTempFile;
use uuid::Uuid;
use yunheshiguang_sdk::device::DeviceShell;
use yunheshiguang_sdk::scripts::{
    ScriptEngine, ScriptEngineInfo, ScriptLanguage, ScriptOutcome, ScriptRequest,
};

use crate::workspace::contract::UiText;
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

pub const SHELL_ENGINE_ID: &str = "core.shell";
const REMOTE_DIRECTORY: &str = "/data/local/tmp/yunheshiguang-scripts";
const EXIT_MARKER: &str = "__yunheshiguang_exit__=";
const TIMEOUT: Duration = Duration::from_secs(300);
const DEVICE_TIMEOUT: Duration = Duration::from_secs(290);
pub const MAX_OUTPUT_CHARS: usize = 16 * 1024;
const OUTPUT_TAIL_BYTES: usize = MAX_OUTPUT_CHARS * 4;

fn remote_stem(id: &str) -> String {
    let name = id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(64)
        .collect::<String>();

    format!(
        "{REMOTE_DIRECTORY}/{}-{}",
        if name.is_empty() { "script" } else { &name },
        Uuid::now_v7().simple()
    )
}

pub fn parse_output(stdout: &str) -> Option<ScriptOutcome> {
    let index = stdout.rfind(EXIT_MARKER)?;
    let exit_code = stdout[index + EXIT_MARKER.len()..]
        .trim()
        .parse::<i32>()
        .ok()?;
    let body = stdout[..index].trim_end_matches(['\n', '\r']);
    let characters = body.chars().count();
    let output = if characters > MAX_OUTPUT_CHARS {
        body.chars().skip(characters - MAX_OUTPUT_CHARS).collect()
    } else {
        body.to_string()
    };

    Some(ScriptOutcome::finished(exit_code, output).with_truncated(characters > MAX_OUTPUT_CHARS))
}

pub struct ShellEngine;

impl ShellEngine {
    fn upload(device: &dyn DeviceShell, script: &ScriptRequest<'_>) -> Result<String, UiText> {
        let failed = |message: String| {
            ui_text_with_params(SCRIPT_UPLOAD_FAILED_KEY, "", [("message", message)])
        };
        let mut local = NamedTempFile::new().map_err(|error| failed(error.to_string()))?;
        local
            .write_all(script.body.replace("\r\n", "\n").as_bytes())
            .map_err(|error| failed(error.to_string()))?;
        let stem = remote_stem(script.id);

        device.shell(&["mkdir", "-p", REMOTE_DIRECTORY])?;
        device.push(local.path(), &format!("{stem}.sh"))?;
        Ok(stem)
    }
}

impl ScriptEngine for ShellEngine {
    fn describe(&self) -> ScriptEngineInfo {
        ScriptEngineInfo {
            id: SHELL_ENGINE_ID.to_string(),
            label: ui_text(SCRIPT_SHELL_LABEL_KEY).into(),
            description: ui_text(SCRIPT_SHELL_DESCRIPTION_KEY).into(),
            languages: vec![ScriptLanguage::Shell.into()],
            captures_output: true,
            setup: Vec::new(),
            ..Default::default()
        }
    }

    fn run(
        &self,
        device: &dyn DeviceShell,
        script: &ScriptRequest<'_>,
    ) -> Result<ScriptOutcome, UiText> {
        let stem = Self::upload(device, script)?;
        let limit = DEVICE_TIMEOUT.as_secs();
        let wrapper = format!(
            "if command -v timeout >/dev/null 2>&1; then timeout -s KILL {limit} sh {stem}.sh; else sh {stem}.sh; fi > {stem}.out 2>&1; code=$?; tail -c {OUTPUT_TAIL_BYTES} {stem}.out; rm -f {stem}.sh {stem}.out; printf '\\n%s%d\\n' '{EXIT_MARKER}' \"$code\""
        );
        let output = device.shell_with_timeout(&["sh", "-c", &wrapper], TIMEOUT)?;

        parse_output(&output.stdout).ok_or_else(|| ui_text(SCRIPT_NO_EXIT_CODE_KEY))
    }
}
