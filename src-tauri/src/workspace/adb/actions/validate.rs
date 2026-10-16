use std::path::PathBuf;

pub fn required_path(value: &str, field: &str) -> Result<PathBuf, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{field} is required"))
    } else {
        Ok(PathBuf::from(value))
    }
}

pub fn required_file(value: &str, field: &str) -> Result<PathBuf, String> {
    let path = required_path(value, field)?;
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("{field} does not exist or is not a file"))
    }
}

pub(super) fn required_text<'a>(value: &'a str, field: &str) -> Result<&'a str, String> {
    if value.trim().is_empty() {
        Err(format!("{field} is required"))
    } else {
        Ok(value)
    }
}

pub const MAX_INPUT_TEXT_BYTES: usize = 3000;

pub fn validate_input_text(value: &str, field: &str) -> Result<String, String> {
    let value = required_text(value, field)?;
    if value.contains('\0') {
        return Err(format!("{field} contains a NUL character"));
    }
    if value.len() > MAX_INPUT_TEXT_BYTES {
        return Err(format!(
            "{field} is longer than {MAX_INPUT_TEXT_BYTES} bytes"
        ));
    }
    Ok(value.replace("\r\n", "\n"))
}

pub fn validate_script_id(value: &str) -> Result<String, String> {
    let value = required_text(value, "script")?.trim();
    let valid = value.len() <= 128
        && value
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));

    if valid {
        Ok(value.to_string())
    } else {
        Err("script id is invalid".to_string())
    }
}

pub fn validate_package_name(value: &str) -> Result<String, String> {
    let value = required_text(value, "package name")?;
    let segments = value.split('.').collect::<Vec<_>>();
    if segments.len() < 2
        || !segments
            .iter()
            .all(|segment| valid_package_segment(segment))
    {
        return Err("package name is invalid".to_string());
    }
    Ok(value.to_string())
}

fn valid_package_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_alphabetic())
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
