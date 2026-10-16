use crate::contract::yunheshiguang::workspace::v1::{
    PiiClass, SearchableText, UiText, VisibilityMode,
};

pub fn ui_text(key: impl Into<String>) -> UiText {
    UiText {
        key: key.into(),
        params: Default::default(),
        fallback: String::new(),
        ..Default::default()
    }
}

pub fn ui_text_with_params(
    key: impl Into<String>,
    fallback: impl Into<String>,
    params: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
) -> UiText {
    UiText {
        key: key.into(),
        params: params
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
        fallback: fallback.into(),
        ..Default::default()
    }
}

pub fn literal_text(value: impl Into<String>) -> UiText {
    UiText {
        key: String::new(),
        params: Default::default(),
        fallback: value.into(),
        ..Default::default()
    }
}

pub fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn mask_middle(value: &str, visible_prefix: usize, visible_suffix: usize) -> String {
    let characters: Vec<char> = value.chars().collect();

    if characters.len() <= visible_prefix + visible_suffix + 1 {
        return characters
            .iter()
            .enumerate()
            .map(|(index, character)| if index == 0 { *character } else { '*' })
            .collect();
    }

    let prefix: String = characters.iter().take(visible_prefix).collect();
    let suffix: String = characters
        .iter()
        .skip(characters.len().saturating_sub(visible_suffix))
        .collect();
    let masked_len = characters
        .len()
        .saturating_sub(visible_prefix + visible_suffix)
        .min(6);

    format!("{prefix}{}{suffix}", "*".repeat(masked_len))
}

fn base_text(
    value: String,
    pii_class: PiiClass,
    visibility: VisibilityMode,
    redacted: String,
) -> SearchableText {
    SearchableText {
        canonical_value: normalize(&value),
        display_value: value,
        aliases: Vec::new(),
        redacted_value: redacted,
        pii_class: pii_class.into(),
        visibility: visibility.into(),
        searchable: true,
        ..Default::default()
    }
}

pub fn public_text(value: impl Into<String>) -> SearchableText {
    let value = value.into();
    base_text(
        value.clone(),
        PiiClass::None,
        VisibilityMode::Visible,
        value,
    )
}

pub fn pii_text(value: impl Into<String>, pii_class: PiiClass) -> SearchableText {
    let value = value.into();
    let redacted = mask_middle(&value, 2, 2);
    base_text(value, pii_class, VisibilityMode::Searchable, redacted)
}

pub fn note_text(value: impl Into<String>) -> SearchableText {
    base_text(
        value.into(),
        PiiClass::Note,
        VisibilityMode::Searchable,
        "******".to_string(),
    )
}

pub fn mask_device_id(device_id: &str) -> String {
    let characters = device_id.chars().collect::<Vec<_>>();

    if characters.len() <= 4 {
        return "****".to_string();
    }

    let suffix = characters[characters.len() - 4..]
        .iter()
        .collect::<String>();

    format!("***{suffix}")
}

#[cfg(test)]
mod tests {
    use super::{
        literal_text, mask_device_id, mask_middle, normalize, note_text, pii_text,
        ui_text_with_params,
    };
    use crate::contract::yunheshiguang::workspace::v1::PiiClass;

    #[test]
    fn masks_a_note_without_revealing_any_of_it() {
        let text = note_text("张三的账号 13800138000");

        assert_eq!(text.redacted_value, "******");
        assert_eq!(text.pii_class, PiiClass::Note);
        assert_eq!(text.display_value, "张三的账号 13800138000");
    }

    #[test]
    fn shows_a_literal_as_it_is() {
        let text = literal_text("A 区货架");

        assert!(text.key.is_empty());
        assert_eq!(text.fallback, "A 区货架");
    }

    #[test]
    fn normalizes_case_and_whitespace() {
        assert_eq!(normalize("  AbC  "), "abc");
    }

    #[test]
    fn masks_middle_portion_without_overexposing() {
        assert_eq!(mask_middle("1234567890", 2, 2), "12******90");
    }

    #[test]
    fn keeps_a_sensitive_value_out_of_the_fields_a_masked_search_reads() {
        let text = pii_text("R58M20ABCDE", PiiClass::DeviceIdentifier);

        assert!(text.aliases.is_empty());
        assert!(!text.redacted_value.to_lowercase().contains("r58m20abcde"));
        assert_eq!(text.display_value, "R58M20ABCDE");
        assert_eq!(text.canonical_value, "r58m20abcde");
    }

    #[test]
    fn keeps_only_the_last_four_characters_of_a_device_id() {
        assert_eq!(mask_device_id("0123456789"), "***6789");
        assert_eq!(mask_device_id("192.168.1.20:5555"), "***5555");
        assert_eq!(mask_device_id("1234"), "****");
    }

    #[test]
    fn carries_params_by_name() {
        let text = ui_text_with_params("a.b", "", [("device", "x")]);

        assert_eq!(text.key, "a.b");
        assert_eq!(text.params["device"], "x");
    }
}
