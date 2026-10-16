use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use sha2::{Digest, Sha256};
use yunheshiguang_sdk::device::{DeviceShell, ShellOutput};
use yunheshiguang_sdk::input::{InputChannel, InputChannelInfo, InputKind, InputRequest};
use yunheshiguang_sdk::vendor::DeviceIdentity;

use super::router::InputRouter;
use super::yadb::{
    REMOTE_PATH, YADB, YADB_CHANNEL_ID, YADB_SHA256, YadbChannel, install_host_copy,
};
use crate::workspace::contract::UiText;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

#[derive(Default)]
struct ScriptedShell {
    size: Option<String>,
    output: String,
    calls: RefCell<Vec<Vec<String>>>,
    pushes: RefCell<Vec<String>>,
}

impl ScriptedShell {
    fn answering(output: &str) -> Self {
        Self {
            output: output.to_string(),
            ..Self::default()
        }
    }

    fn last_call(&self) -> Vec<String> {
        self.calls.borrow().last().cloned().unwrap_or_default()
    }
}

impl DeviceShell for ScriptedShell {
    fn identity(&self) -> Option<&DeviceIdentity> {
        None
    }

    fn shell(&self, args: &[&str]) -> Result<ShellOutput, UiText> {
        self.calls
            .borrow_mut()
            .push(args.iter().map(ToString::to_string).collect());

        if args.first() == Some(&"stat") {
            return self
                .size
                .clone()
                .map(ShellOutput::new)
                .ok_or_else(|| ui_text(DEVICE_SHELL_FAILED_KEY));
        }

        Ok(ShellOutput::new(self.output.clone()))
    }

    fn push(&self, _local: &Path, remote: &str) -> Result<(), UiText> {
        self.pushes.borrow_mut().push(remote.to_string());
        Ok(())
    }
}

fn yadb() -> YadbChannel {
    YadbChannel::new(PathBuf::from("/tmp/yadb-host-copy"))
}

#[test]
fn embeds_exactly_the_pinned_yadb_release() {
    let digest = Sha256::digest(YADB)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    assert_eq!(digest, YADB_SHA256);
}

#[test]
fn pushes_yadb_when_missing_and_types_through_the_clipboard() {
    let device = ScriptedShell::answering("Copy text:true\n");

    let result = yadb().deliver(&device, &InputRequest::Text("你好\n世界"));

    assert_eq!(result, Ok(()));
    assert_eq!(*device.pushes.borrow(), [REMOTE_PATH]);
    let call = device.last_call();
    assert_eq!(call[0], "app_process");
    assert_eq!(call[4], "-keyboard");
    assert_eq!(call[5], "你好\\n世界");
}

#[test]
fn does_not_push_again_when_the_device_already_has_this_yadb() {
    let device = ScriptedShell {
        size: Some(format!("{}\n", YADB.len())),
        output: "Copy text:true".to_string(),
        ..ScriptedShell::default()
    };

    assert_eq!(yadb().deliver(&device, &InputRequest::Text("hi")), Ok(()));
    assert!(device.pushes.borrow().is_empty());
}

#[test]
fn writes_the_clipboard_with_line_breaks_kept() {
    let device = ScriptedShell::answering("");

    assert_eq!(
        yadb().deliver(&device, &InputRequest::Clipboard("第一行\n第二行")),
        Ok(())
    );
    assert_eq!(device.last_call()[4], "-writeClipboard");
    assert_eq!(device.last_call()[5], "第一行\n第二行");
}

#[test]
fn reports_what_yadb_printed_when_it_failed() {
    let device = ScriptedShell::answering(
        "MainException: java.lang.SecurityException: Injecting input events requires the \
         caller to have the INJECT_EVENTS permission.\n\tat android.os.Parcel",
    );

    let failure = yadb()
        .deliver(&device, &InputRequest::Text("hi"))
        .unwrap_err();

    assert_eq!(failure.key, INPUT_YADB_FAILED_KEY);
    assert!(failure.params["message"].contains("INJECT_EVENTS"));
    assert!(!failure.params["message"].contains("android.os.Parcel"));
}

#[test]
fn treats_typing_that_yadb_did_not_confirm_as_failed() {
    let device = ScriptedShell::answering("");

    assert_eq!(
        yadb().deliver(&device, &InputRequest::Text("hi")),
        Err(ui_text(INPUT_YADB_UNCONFIRMED_KEY))
    );
}

#[test]
fn writes_the_host_copy_once_and_repairs_a_changed_one() {
    let directory = tempfile::tempdir().unwrap();

    let path = install_host_copy(directory.path()).unwrap();
    assert_eq!(fs::read(&path).unwrap(), YADB);

    fs::write(&path, b"truncated").unwrap();
    assert_eq!(install_host_copy(directory.path()).unwrap(), path);
    assert_eq!(fs::read(&path).unwrap(), YADB);
}

struct FakeChannel {
    id: &'static str,
    kinds: Vec<InputKind>,
    result: Result<(), UiText>,
    delivered: AtomicUsize,
}

impl FakeChannel {
    fn new(id: &'static str, kinds: Vec<InputKind>, result: Result<(), UiText>) -> Self {
        Self {
            id,
            kinds,
            result,
            delivered: AtomicUsize::new(0),
        }
    }
}

impl InputChannel for FakeChannel {
    fn describe(&self) -> InputChannelInfo {
        InputChannelInfo {
            id: self.id.to_string(),
            kinds: self.kinds.iter().map(|kind| (*kind).into()).collect(),
            ..InputChannelInfo::default()
        }
    }

    fn deliver(
        &self,
        _device: &dyn DeviceShell,
        _request: &InputRequest<'_>,
    ) -> Result<(), UiText> {
        self.delivered.fetch_add(1, Ordering::SeqCst);
        self.result.clone()
    }
}

#[test]
fn asks_extension_channels_first_and_falls_through_by_kind() {
    let router = InputRouter::new(
        vec![Box::new(FakeChannel::new(
            "extension.ime",
            vec![InputKind::Text],
            Ok(()),
        ))],
        vec![Box::new(FakeChannel::new(
            YADB_CHANNEL_ID,
            vec![InputKind::Text, InputKind::Clipboard],
            Ok(()),
        ))],
    );
    let device = ScriptedShell::default();

    let typed = router.deliver(&device, &InputRequest::Text("hi"));
    let copied = router.deliver(&device, &InputRequest::Clipboard("hi"));

    assert_eq!(typed.route.unwrap().id, "extension.ime");
    assert_eq!(copied.route.unwrap().id, YADB_CHANNEL_ID);
    assert_eq!(
        router
            .channels()
            .channels
            .iter()
            .map(|channel| channel.id.as_str())
            .collect::<Vec<_>>(),
        ["extension.ime", YADB_CHANNEL_ID]
    );
}

#[test]
fn does_not_retry_a_failed_delivery_on_another_channel() {
    let failure = ui_text(INPUT_YADB_UNCONFIRMED_KEY);
    let router = InputRouter::new(
        vec![Box::new(FakeChannel::new(
            "extension.ime",
            vec![InputKind::Text],
            Err(failure.clone()),
        ))],
        vec![Box::new(FakeChannel::new(
            YADB_CHANNEL_ID,
            vec![InputKind::Text],
            Ok(()),
        ))],
    );

    let delivery = router.deliver(&ScriptedShell::default(), &InputRequest::Text("hi"));

    assert_eq!(delivery.result, Err(failure));
    assert_eq!(delivery.route.unwrap().id, "extension.ime");
}

#[test]
fn says_so_when_no_channel_can_serve_the_device() {
    let router = InputRouter::new(
        vec![
            Box::new(FakeChannel::new("a", vec![InputKind::Text], Ok(()))),
            Box::new(FakeChannel::new("a", vec![InputKind::Clipboard], Ok(()))),
        ],
        Vec::new(),
    );

    let delivery = router.deliver(&ScriptedShell::default(), &InputRequest::Clipboard("hi"));

    assert_eq!(delivery.route, None);
    assert_eq!(delivery.result, Err(ui_text(INPUT_NO_CHANNEL_KEY)));
    assert_eq!(router.channels().channels.len(), 1);
}
