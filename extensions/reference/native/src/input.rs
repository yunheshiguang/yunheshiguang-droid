use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::UiText;
use yunheshiguang_sdk::device::{DeviceShell, MAX_SHELL_COMMAND_BYTES};
use yunheshiguang_sdk::input::{InputChannel, InputChannelInfo, InputKind, InputRequest};
use yunheshiguang_sdk::text::{ui_text, ui_text_with_params};

use crate::messages::*;

pub const ADB_KEYBOARD_CHANNEL_ID: &str = "reference.adb-keyboard";
const IME_ID: &str = "com.android.adbkeyboard/.AdbIME";
const BROADCAST_DONE: &str = "Broadcast completed";
const BROADCAST: [&str; 6] = ["am", "broadcast", "-a", "ADB_INPUT_B64", "--es", "msg"];

fn fits(text: &str) -> bool {
    let prefix = BROADCAST.iter().map(|part| part.len() + 1).sum::<usize>();

    prefix + text.len().div_ceil(3) * 4 <= MAX_SHELL_COMMAND_BYTES
}

pub struct AdbKeyboardChannel;

impl InputChannel for AdbKeyboardChannel {
    fn describe(&self) -> InputChannelInfo {
        InputChannelInfo {
            id: ADB_KEYBOARD_CHANNEL_ID.to_string(),
            label: ui_text(REFERENCE_INPUT_LABEL_KEY).into(),
            description: ui_text(REFERENCE_INPUT_DESCRIPTION_KEY).into(),
            kinds: vec![InputKind::Text.into()],
            setup: vec![
                ui_text(REFERENCE_INPUT_SETUP_INSTALL_KEY),
                ui_text_with_params(REFERENCE_INPUT_SETUP_SELECT_KEY, "", [("ime", IME_ID)]),
            ],
            ..Default::default()
        }
    }

    fn accepts(&self, device: &dyn DeviceShell, request: &InputRequest<'_>) -> bool {
        matches!(request, InputRequest::Text(text) if fits(text))
            && device
                .shell(&["settings", "get", "secure", "default_input_method"])
                .is_ok_and(|output| output.stdout.trim() == IME_ID)
    }

    fn deliver(&self, device: &dyn DeviceShell, request: &InputRequest<'_>) -> Result<(), UiText> {
        let InputRequest::Text(text) = request else {
            return Err(ui_text(REFERENCE_INPUT_TEXT_ONLY_KEY));
        };
        let encoded = STANDARD.encode(text);
        let mut command = BROADCAST.to_vec();
        command.push(&encoded);
        let output = device.shell(&command)?;

        if output.stdout.contains(BROADCAST_DONE) {
            Ok(())
        } else {
            Err(ui_text_with_params(
                REFERENCE_INPUT_FAILED_KEY,
                "",
                [("message", output.stdout.trim().to_string())],
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::Path;

    use yunheshiguang_sdk::device::ShellOutput;
    use yunheshiguang_sdk::vendor::DeviceIdentity;

    use super::*;

    struct Device {
        ime: &'static str,
        output: &'static str,
        calls: RefCell<Vec<Vec<String>>>,
    }

    impl Device {
        fn with_ime(ime: &'static str) -> Self {
            Self {
                ime,
                output: "Broadcasting: Intent { act=ADB_INPUT_B64 }\nBroadcast completed: result=0",
                calls: RefCell::new(Vec::new()),
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

            Ok(ShellOutput::new(if args[0] == "settings" {
                self.ime
            } else {
                self.output
            }))
        }

        fn push(&self, _local: &Path, _remote: &str) -> Result<(), UiText> {
            Ok(())
        }
    }

    #[test]
    fn takes_only_typing_and_only_while_it_is_the_current_keyboard() {
        let selected = Device::with_ime("com.android.adbkeyboard/.AdbIME\n");
        let other = Device::with_ime("com.sohu.inputmethod.sogou/.SogouIME");

        assert!(AdbKeyboardChannel.accepts(&selected, &InputRequest::Text("你好")));
        assert!(!AdbKeyboardChannel.accepts(&selected, &InputRequest::Clipboard("你好")));
        assert!(!AdbKeyboardChannel.accepts(&other, &InputRequest::Text("你好")));
    }

    #[test]
    fn leaves_text_too_long_for_one_broadcast_to_another_channel() {
        let selected = Device::with_ime(IME_ID);
        let longest = "x".repeat(2970);

        assert!(AdbKeyboardChannel.accepts(&selected, &InputRequest::Text(&longest)));
        assert!(
            !AdbKeyboardChannel.accepts(&selected, &InputRequest::Text(&format!("{longest}xxx")))
        );
    }

    #[test]
    fn broadcasts_the_text_as_base64() {
        let device = Device::with_ime(IME_ID);

        assert_eq!(
            AdbKeyboardChannel.deliver(&device, &InputRequest::Text("你好 'x'")),
            Ok(())
        );
        let call = device.calls.borrow().last().cloned().unwrap();
        assert_eq!(
            call[..6],
            ["am", "broadcast", "-a", "ADB_INPUT_B64", "--es", "msg"]
        );
        assert_eq!(STANDARD.decode(&call[6]).unwrap(), "你好 'x'".as_bytes());
    }

    #[test]
    fn fails_when_the_broadcast_did_not_go_out() {
        let device = Device {
            output: "Error: Activity manager is not running",
            ..Device::with_ime(IME_ID)
        };

        let failure = AdbKeyboardChannel
            .deliver(&device, &InputRequest::Text("hi"))
            .unwrap_err();

        assert_eq!(failure.key, REFERENCE_INPUT_FAILED_KEY);
        assert!(failure.params["message"].contains("Activity manager"));
    }
}
