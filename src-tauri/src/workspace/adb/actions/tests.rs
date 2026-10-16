use std::io::Write;

use tempfile::NamedTempFile;

use super::artifact::validate_mp4;
use super::*;

#[test]
fn keeps_input_text_as_written_and_rejects_only_what_cannot_be_sent() {
    assert_eq!(
        validate_input_text("你好, it's 5 o'clock; ok?", "input text").as_deref(),
        Ok("你好, it's 5 o'clock; ok?")
    );
    assert_eq!(
        validate_input_text("line 1\r\nline 2", "input text").as_deref(),
        Ok("line 1\nline 2")
    );
    assert!(validate_input_text("   ", "input text").is_err());
    assert!(validate_input_text("a\0b", "input text").is_err());
    assert!(validate_input_text(&"字".repeat(1001), "input text").is_err());
}

#[test]
fn validates_android_package_names() {
    assert!(validate_package_name("com.example.app").is_ok());
    for invalid in [
        "not-a-package",
        "1com.example",
        "com..example",
        "com.x;reboot",
    ] {
        assert!(
            validate_package_name(invalid).is_err(),
            "{invalid:?} should be refused"
        );
    }
}

#[test]
fn validates_script_ids_as_document_ids() {
    assert_eq!(
        validate_script_id("script-01_a").as_deref(),
        Ok("script-01_a")
    );

    for invalid in ["", "../etc", "-flag", "a b", "a;reboot", &"a".repeat(129)] {
        assert!(
            validate_script_id(invalid).is_err(),
            "{invalid:?} should be refused"
        );
    }
}

#[test]
fn requires_the_success_line_when_a_command_expects_one() {
    let plain = AdbCommand::new(
        ["shell", "am", "force-stop", "com.example"],
        TRANSFER_POLICY,
    );
    let expecting = plain.clone().expecting("Success");

    assert!(plain.check("").is_ok());
    assert!(expecting.check("Success\n").is_ok());
    assert!(expecting.check("  Success  \r\n").is_ok());

    let failed = expecting.check("Failed\n").unwrap_err();
    assert_eq!(failed.message, "Failed");
    assert!(
        expecting
            .check("Failure [DELETE_FAILED_INTERNAL_ERROR]")
            .is_err()
    );
}

#[test]
fn validates_mp4_container_header() {
    let mut valid = NamedTempFile::new().expect("temporary recording should exist");
    valid
        .as_file_mut()
        .write_all(b"\0\0\0\x18ftyp")
        .expect("header should save");
    let mut invalid = NamedTempFile::new().expect("temporary recording should exist");
    invalid
        .as_file_mut()
        .write_all(b"not-mp4!")
        .expect("header should save");

    assert!(validate_mp4(valid.path()).is_ok());
    assert!(validate_mp4(invalid.path()).is_err());
}
