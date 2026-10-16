use crate::workspace::contract::{ScrcpyCaptureOrientation, ScrcpyConfig, ScrcpyDiagnosticCode};
use yunheshiguang_contract::KnownEnum;

use super::diagnostics::ScrcpyProblem;

const MAX_TITLE_CHARS: usize = 128;

pub fn default_config() -> ScrcpyConfig {
    ScrcpyConfig {
        video_enabled: true,
        audio_enabled: true,
        control_enabled: true,
        ..ScrcpyConfig::default()
    }
}

pub fn validate_serial(serial: &str) -> Result<&str, ScrcpyProblem> {
    let valid = !serial.is_empty()
        && serial.len() <= 256
        && serial
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._:-[]%".contains(character));

    if valid {
        Ok(serial)
    } else {
        Err(ScrcpyProblem::new(ScrcpyDiagnosticCode::InvalidSerial, ""))
    }
}

fn invalid(message: &str) -> ScrcpyProblem {
    ScrcpyProblem::new(ScrcpyDiagnosticCode::InvalidArgument, message)
}

fn crop(value: &str) -> Result<String, ScrcpyProblem> {
    let parts = value.split(':').collect::<Vec<_>>();
    let numeric = parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= 6 && part.bytes().all(|b| b.is_ascii_digit())
        });

    if numeric {
        Ok(format!("--crop={value}"))
    } else {
        Err(invalid("crop must be width:height:x:y"))
    }
}

fn orientation(value: i32) -> Option<&'static str> {
    match ScrcpyCaptureOrientation::known(value)? {
        ScrcpyCaptureOrientation::Unspecified | ScrcpyCaptureOrientation::Rotation0 => None,
        ScrcpyCaptureOrientation::Rotation90 => Some("--capture-orientation=90"),
        ScrcpyCaptureOrientation::Rotation180 => Some("--capture-orientation=180"),
        ScrcpyCaptureOrientation::Rotation270 => Some("--capture-orientation=270"),
    }
}

pub fn launch_args(serial: &str, config: &ScrcpyConfig) -> Result<Vec<String>, ScrcpyProblem> {
    let serial = validate_serial(serial)?;
    if !config.video_enabled && !config.audio_enabled {
        return Err(invalid(
            "turn on video or audio, or there is nothing to mirror",
        ));
    }

    let mut args = vec![format!("--serial={serial}")];
    let control = config.control_enabled;
    let flags = [
        (!config.video_enabled, "--no-video"),
        (!config.audio_enabled, "--no-audio"),
        (!control, "--no-control"),
        (control && config.turn_screen_off, "--turn-screen-off"),
        (control && config.stay_awake, "--stay-awake"),
        (control && config.show_touches, "--show-touches"),
        (config.keep_active, "--keep-active"),
        (config.fullscreen, "--fullscreen"),
        (config.always_on_top, "--always-on-top"),
    ];
    args.extend(
        flags
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, flag)| flag.to_string()),
    );

    if config.max_size > 0 {
        args.push(format!("--max-size={}", config.max_size));
    }
    if config.video_bit_rate_bps > 0 {
        args.push(format!("--video-bit-rate={}", config.video_bit_rate_bps));
    }
    if config.max_fps > 0 {
        args.push(format!("--max-fps={}", config.max_fps));
    }

    let title = config.window_title.trim();
    if !title.is_empty() {
        if title.chars().count() > MAX_TITLE_CHARS || title.chars().any(char::is_control) {
            return Err(invalid(
                "the window title is too long or has control characters",
            ));
        }
        args.push(format!("--window-title={title}"));
    }

    let cropped = config.crop.trim();
    if !cropped.is_empty() {
        args.push(crop(cropped)?);
    }
    if let Some(flag) = orientation(config.capture_orientation.to_i32()) {
        args.push(flag.to_string());
    }

    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_with_scrcpy_defaults_unless_told_otherwise() {
        assert_eq!(
            launch_args("emulator-5554", &default_config()).unwrap(),
            ["--serial=emulator-5554"]
        );
    }

    #[test]
    fn turns_each_setting_into_its_scrcpy_option() {
        let config = ScrcpyConfig {
            audio_enabled: false,
            control_enabled: false,
            max_size: 1024,
            video_bit_rate_bps: 4_000_000,
            max_fps: 30,
            turn_screen_off: true,
            stay_awake: true,
            keep_active: true,
            window_title: "  Shelf 3  ".to_string(),
            crop: "1080:1920:0:0".to_string(),
            capture_orientation: ScrcpyCaptureOrientation::Rotation90.into(),
            ..default_config()
        };

        assert_eq!(
            launch_args("192.168.1.20:5555", &config).unwrap(),
            [
                "--serial=192.168.1.20:5555",
                "--no-audio",
                "--no-control",
                "--keep-active",
                "--max-size=1024",
                "--video-bit-rate=4000000",
                "--max-fps=30",
                "--window-title=Shelf 3",
                "--crop=1080:1920:0:0",
                "--capture-orientation=90",
            ]
        );
    }

    #[test]
    fn refuses_what_scrcpy_could_misread() {
        let code =
            |serial: &str, config: ScrcpyConfig| launch_args(serial, &config).unwrap_err().code;

        assert_eq!(
            code("", default_config()),
            ScrcpyDiagnosticCode::InvalidSerial
        );
        assert_eq!(
            code("abc --record=/tmp/x", default_config()),
            ScrcpyDiagnosticCode::InvalidSerial
        );
        assert_eq!(
            code(
                "abc",
                ScrcpyConfig {
                    video_enabled: false,
                    audio_enabled: false,
                    ..default_config()
                }
            ),
            ScrcpyDiagnosticCode::InvalidArgument
        );
        assert_eq!(
            code(
                "abc",
                ScrcpyConfig {
                    crop: "1080:1920:0".to_string(),
                    ..default_config()
                }
            ),
            ScrcpyDiagnosticCode::InvalidArgument
        );
        assert_eq!(
            code(
                "abc",
                ScrcpyConfig {
                    window_title: "a\nb".to_string(),
                    ..default_config()
                }
            ),
            ScrcpyDiagnosticCode::InvalidArgument
        );
    }
}
