use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use yunheshiguang_sdk::documents::DocumentId;
use yunheshiguang_sdk::fs::write_atomic;

use crate::workspace::contract::{PackageManifest, TemplatePackage, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::signing::{Signed, SigningKeys, verify};

pub const MAX_PACKAGE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_ID_CHARS: usize = 100;
const MAX_NAME_CHARS: usize = 80;
const MAX_TEXT_CHARS: usize = 2_000;

fn canonical(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            out.push('{');
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                canonical(&map[key], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                canonical(item, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

fn canonical_without(package: &Value, fields: &[&str]) -> String {
    let mut value = package.clone();
    if let Some(object) = value.as_object_mut() {
        for field in fields {
            object.remove(*field);
        }
    }
    let mut text = String::new();
    canonical(&value, &mut text);
    text
}

pub fn digest_of(package: &Value) -> String {
    Sha256::digest(canonical_without(package, &["digest"]).as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn signed_text(package: &Value) -> String {
    canonical_without(package, &["digest", "signature"])
}

fn read_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        PACKAGES_READ_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

pub fn storage_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        PACKAGES_STORAGE_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

pub struct Loaded {
    pub package: TemplatePackage,
    pub verified: bool,
    pub signer: Option<Signed>,
}

pub fn parse_package(text: &str) -> Result<Loaded, UiText> {
    let unreadable = |error: serde_json::Error| {
        ui_text_with_params(
            PACKAGES_UNREADABLE_KEY,
            "",
            [("message", error.to_string())],
        )
    };
    let value = serde_json::from_str::<Value>(text).map_err(unreadable)?;
    let recorded = value
        .get("digest")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !recorded.is_empty() && recorded != digest_of(&value) {
        return Err(ui_text(PACKAGES_DIGEST_MISMATCH_KEY));
    }
    let signer = verify(&value)?;

    Ok(Loaded {
        package: yunheshiguang_contract::json::from_value(value).map_err(unreadable)?,
        verified: false,
        signer,
    })
}

fn check_size(size: u64) -> Result<(), UiText> {
    if size > MAX_PACKAGE_BYTES {
        Err(ui_text_with_params(
            PACKAGES_TOO_LARGE_KEY,
            "",
            [("max", (MAX_PACKAGE_BYTES / 1024 / 1024).to_string())],
        ))
    } else {
        Ok(())
    }
}

pub fn read_package(path: &Path) -> Result<Loaded, UiText> {
    let metadata = std::fs::metadata(path).map_err(read_failed)?;
    if !metadata.is_file() {
        return Err(read_failed("not a regular file"));
    }
    check_size(metadata.len())?;
    let mut text = String::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(MAX_PACKAGE_BYTES + 1).read_to_string(&mut text))
        .map_err(read_failed)?;

    read_package_text(&text)
}

pub fn read_package_text(text: &str) -> Result<Loaded, UiText> {
    check_size(text.len() as u64)?;

    parse_package(text)
}

pub fn write_package(
    path: &Path,
    package: &TemplatePackage,
    keys: &SigningKeys,
    signer: &str,
) -> Result<(), UiText> {
    if !path.parent().is_some_and(Path::is_dir) {
        return Err(ui_text(PACKAGES_PATH_INVALID_KEY));
    }
    let mut value = serde_json::to_value(package).map_err(storage_failed)?;
    let signature = serde_json::to_value(keys.sign(&value, signer)?).map_err(storage_failed)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("signature".to_string(), signature);
    }
    let digest = digest_of(&value);
    if let Some(object) = value.as_object_mut() {
        object.insert("digest".to_string(), Value::String(digest));
    }

    let bytes = serde_json::to_vec_pretty(&value).map_err(storage_failed)?;

    write_atomic(path, &bytes).map_err(storage_failed)
}

fn too_long(value: &str, max: usize) -> Option<UiText> {
    (value.chars().count() > max)
        .then(|| ui_text_with_params(PACKAGES_TEXT_TOO_LONG_KEY, "", [("max", max.to_string())]))
}

pub fn check_manifest(manifest: &PackageManifest) -> Result<(), UiText> {
    let id_ok = !manifest.id.is_empty()
        && manifest.id.chars().count() <= MAX_ID_CHARS
        && manifest
            .id
            .starts_with(|first: char| first.is_ascii_alphanumeric())
        && manifest
            .id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character));

    if !id_ok {
        return Err(ui_text(PACKAGES_ID_INVALID_KEY));
    }
    if manifest.name.trim().is_empty() {
        return Err(ui_text(PACKAGES_NAME_REQUIRED_KEY));
    }
    if semver::Version::parse(manifest.version.trim()).is_err() {
        return Err(ui_text(PACKAGES_VERSION_INVALID_KEY));
    }
    if let Some(problem) = too_long(&manifest.name, MAX_NAME_CHARS)
        .or_else(|| too_long(&manifest.author, MAX_NAME_CHARS))
        .or_else(|| too_long(&manifest.description, MAX_TEXT_CHARS))
    {
        return Err(problem);
    }

    Ok(())
}

pub fn check_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<(), UiText> {
    let mut seen = HashSet::new();

    for id in ids {
        if DocumentId::parse(id).is_err() {
            return Err(ui_text_with_params(
                PACKAGES_ITEM_ID_INVALID_KEY,
                "",
                [("id", id.to_string())],
            ));
        }
        if !seen.insert(id) {
            return Err(ui_text_with_params(
                PACKAGES_DUPLICATE_ID_KEY,
                "",
                [("id", id.to_string())],
            ));
        }
    }

    Ok(())
}
