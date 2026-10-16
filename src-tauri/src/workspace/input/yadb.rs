use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;
use yunheshiguang_sdk::device::DeviceShell;
use yunheshiguang_sdk::input::{InputChannel, InputChannelInfo, InputKind, InputRequest};

use crate::workspace::contract::UiText;
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::router::plain_message;

pub const YADB_CHANNEL_ID: &str = "core.yadb";
pub(super) const YADB: &[u8] = include_bytes!("../../../assets/yadb/yadb");
#[cfg(test)]
pub(super) const YADB_SHA256: &str =
    "f1d12e49c94fd227b900b7cfcf74ca9dcff57dbf4aa3c43591ebe742306b1796";
pub(super) const REMOTE_PATH: &str = "/data/local/tmp/yunheshiguang-yadb-1.1.2";
const HOST_FILE: &str = "yunheshiguang-yadb-1.1.2";
const MAIN_CLASS: &str = "com.ysbing.yadb.Main";
const TYPED: &str = "Copy text:true";

pub fn install_host_copy(directory: &Path) -> io::Result<PathBuf> {
    let path = directory.join(HOST_FILE);

    if fs::read(&path).is_ok_and(|existing| existing == YADB) {
        return Ok(path);
    }

    fs::create_dir_all(directory)?;
    let mut temporary = NamedTempFile::new_in(directory)?;
    temporary.write_all(YADB)?;
    temporary.persist(&path).map_err(|error| error.error)?;

    Ok(path)
}

fn exception_line(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.contains("Exception"))
}

pub struct YadbChannel {
    host_copy: PathBuf,
}

impl YadbChannel {
    pub fn new(host_copy: PathBuf) -> Self {
        Self { host_copy }
    }

    fn ensure_pushed(&self, device: &dyn DeviceShell) -> Result<(), UiText> {
        let present = device
            .shell(&["stat", "-c", "%s", REMOTE_PATH])
            .is_ok_and(|output| output.stdout.trim() == YADB.len().to_string());

        if present {
            return Ok(());
        }

        device
            .push(&self.host_copy, REMOTE_PATH)
            .map_err(|failure| {
                ui_text_with_params(
                    INPUT_YADB_PUSH_FAILED_KEY,
                    "",
                    [("message", plain_message(&failure))],
                )
            })
    }

    fn run(&self, device: &dyn DeviceShell, command: &str, text: &str) -> Result<String, UiText> {
        let classpath = format!("-Djava.class.path={REMOTE_PATH}");

        device
            .shell(&[
                "app_process",
                &classpath,
                "/data/local/tmp",
                MAIN_CLASS,
                command,
                text,
            ])
            .map(|output| output.stdout)
    }
}

impl InputChannel for YadbChannel {
    fn describe(&self) -> InputChannelInfo {
        InputChannelInfo {
            id: YADB_CHANNEL_ID.to_string(),
            label: ui_text(INPUT_YADB_LABEL_KEY).into(),
            description: ui_text(INPUT_YADB_DESCRIPTION_KEY).into(),
            kinds: vec![InputKind::Text.into(), InputKind::Clipboard.into()],
            setup: Vec::new(),
            ..Default::default()
        }
    }

    fn deliver(&self, device: &dyn DeviceShell, request: &InputRequest<'_>) -> Result<(), UiText> {
        self.ensure_pushed(device)?;

        let stdout = match request {
            InputRequest::Text(text) => self.run(device, "-keyboard", &text.replace('\n', "\\n")),
            InputRequest::Clipboard(text) => self.run(device, "-writeClipboard", text),
            _ => return Err(ui_text(INPUT_NO_CHANNEL_KEY)),
        }?;

        if let Some(line) = exception_line(&stdout) {
            return Err(ui_text_with_params(
                INPUT_YADB_FAILED_KEY,
                "",
                [("message", line.to_string())],
            ));
        }

        if matches!(request, InputRequest::Text(_)) && !stdout.contains(TYPED) {
            return Err(ui_text(INPUT_YADB_UNCONFIRMED_KEY));
        }

        Ok(())
    }
}
