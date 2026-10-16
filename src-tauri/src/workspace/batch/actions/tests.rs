use std::ffi::OsString;

use crate::workspace::adb::AdbDeviceState;

use super::apps::{CLEAR_APP_DATA, LAUNCH_APP, STOP_APP, UNINSTALL_APP};
use super::command::AdbCommandAction;
use super::*;

fn args_of(action: &AdbCommandAction, request: &BatchActionRequest) -> Vec<String> {
    (action.command)(request)
        .expect("the request should validate")
        .args()
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect()
}

fn app_request(package_name: &str) -> BatchActionRequest {
    BatchActionRequest {
        package_name: package_name.to_string(),
        ..BatchActionRequest::default()
    }
}

#[test]
fn only_recover_connection_runs_on_offline_devices() {
    for kind in super::super::spec::all_action_kinds() {
        let offline = handler(kind)
            .allowed_states()
            .contains(&AdbDeviceState::Offline);

        assert_eq!(
            offline,
            kind == BatchActionKind::RecoverConnection,
            "{kind:?}"
        );
        assert!(
            handler(kind)
                .allowed_states()
                .contains(&AdbDeviceState::Online)
        );
    }
}

#[test]
fn manages_apps_with_the_package_manager_and_activity_manager() {
    let request = app_request("com.example.app");

    assert_eq!(
        args_of(&LAUNCH_APP, &request),
        ["shell", "monkey", "-p", "com.example.app", "1"]
    );
    assert_eq!(
        args_of(&STOP_APP, &request),
        ["shell", "am", "force-stop", "com.example.app"]
    );
    assert_eq!(
        args_of(&CLEAR_APP_DATA, &request),
        ["shell", "pm", "clear", "com.example.app"]
    );
    assert_eq!(
        args_of(&UNINSTALL_APP, &request),
        ["uninstall", "com.example.app"]
    );
}

#[test]
fn refuses_app_actions_for_an_invalid_package_name() {
    for action in [&LAUNCH_APP, &STOP_APP, &CLEAR_APP_DATA, &UNINSTALL_APP] {
        for invalid in ["", "not-a-package", "com.x;reboot", "com.x $(id)"] {
            assert!(
                (action.command)(&app_request(invalid)).is_err(),
                "{:?} accepted {invalid:?}",
                action.spec.kind
            );
        }
    }
}

#[test]
fn pushes_uploads_into_the_download_folder() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("notes.txt");
    std::fs::write(&file, "x").unwrap();
    let request = BatchActionRequest {
        path: file.to_string_lossy().into_owned(),
        ..BatchActionRequest::default()
    };

    let args = (super::transfer::UPLOAD_FILE.command)(&request)
        .unwrap()
        .args()
        .to_vec();

    assert_eq!(
        args,
        [
            OsString::from("push"),
            file.into_os_string(),
            OsString::from("/sdcard/Download/notes.txt")
        ]
    );
    assert!(
        (super::transfer::INSTALL_APK.command)(&BatchActionRequest::default()).is_err(),
        "a missing APK should be refused"
    );
}

#[test]
fn refuses_an_unspecified_action() {
    assert!((super::command::UNSPECIFIED.command)(&BatchActionRequest::default()).is_err());
}
