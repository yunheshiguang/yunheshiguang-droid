use crate::workspace::contract::{BatchActionKind, BatchActionRequest, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};
use yunheshiguang_contract::KnownEnum;

use super::actions::handler;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionRequirement {
    Nothing,
    Text,
    PackageName,
    SourceFile,
    SavePath,
    Script,
}

impl ActionRequirement {
    pub fn uses_host_files(self) -> bool {
        matches!(self, Self::SourceFile | Self::SavePath)
    }
}

pub fn uses_host_files(kind: i32) -> bool {
    spec_for(BatchActionKind::known(kind).unwrap_or(BatchActionKind::Unspecified))
        .requirement
        .uses_host_files()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuccessDetail {
    DeviceCount,
    DeviceCountAndPath,
}

#[derive(Clone, Copy, Debug)]
pub struct BatchActionSpec {
    pub kind: BatchActionKind,
    pub slug: &'static str,
    pub title_key: &'static str,
    pub requirement: ActionRequirement,
    pub success_detail: SuccessDetail,
}

pub fn spec_for(kind: BatchActionKind) -> &'static BatchActionSpec {
    let spec = handler(kind).spec();
    debug_assert_eq!(spec.kind, kind, "{kind:?} is registered to another handler");
    spec
}

impl ActionRequirement {
    fn field(self, request: &mut BatchActionRequest) -> Option<&mut String> {
        match self {
            Self::Nothing => None,
            Self::Text => Some(&mut request.text),
            Self::PackageName => Some(&mut request.package_name),
            Self::SourceFile | Self::SavePath => Some(&mut request.path),
            Self::Script => Some(&mut request.script_id),
        }
    }

    pub fn unmet(self, request: &BatchActionRequest) -> Option<UiText> {
        let (value, key) = match self {
            Self::Nothing => return None,
            Self::Text => (&request.text, VALIDATION_NEED_TEXT_KEY),
            Self::PackageName => (&request.package_name, VALIDATION_NEED_PACKAGE_KEY),
            Self::SourceFile => (&request.path, VALIDATION_NEED_PATH_KEY),
            Self::SavePath => (&request.path, VALIDATION_NEED_SAVE_PATH_KEY),
            Self::Script => (&request.script_id, VALIDATION_NEED_SCRIPT_KEY),
        };

        if value.trim().is_empty() {
            Some(ui_text(key))
        } else {
            None
        }
    }
}

pub fn fill_request(request: &mut BatchActionRequest, kind: BatchActionKind, value: &str) {
    request.kind = kind.into();

    if let Some(field) = spec_for(kind).requirement.field(request) {
        *field = value.to_string();
    }
}

pub fn step_problem(kind: BatchActionKind, value: &str) -> Option<UiText> {
    let mut request = BatchActionRequest::default();
    fill_request(&mut request, kind, value);

    spec_for(kind).requirement.unmet(&request)
}

#[cfg(test)]
pub(super) fn all_action_kinds() -> Vec<BatchActionKind> {
    use yunheshiguang_contract::Enumeration;

    BatchActionKind::values().to_vec()
}

impl SuccessDetail {
    pub fn text(self, request: &BatchActionRequest) -> UiText {
        let count = request.device_ids.len().to_string();

        match self {
            Self::DeviceCount => {
                ui_text_with_params(TASK_DETAIL_COMPLETED_KEY, "", [("count", count)])
            }
            Self::DeviceCountAndPath => ui_text_with_params(
                TASK_DETAIL_SAVED_KEY,
                "",
                [("count", count), ("path", request.path.clone())],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn marks_the_actions_that_read_or_write_files_on_this_computer() {
        let reads_or_writes = |kind: BatchActionKind| super::uses_host_files(kind as i32);

        assert!(reads_or_writes(BatchActionKind::InstallApk));
        assert!(reads_or_writes(BatchActionKind::UploadFile));
        assert!(reads_or_writes(BatchActionKind::Screenshot));
        assert!(reads_or_writes(BatchActionKind::ScreenRecord));
        assert!(!reads_or_writes(BatchActionKind::RunScript));
        assert!(!reads_or_writes(BatchActionKind::InputText));
        assert!(!reads_or_writes(BatchActionKind::Unspecified));
    }

    use super::*;
    use std::collections::HashSet;

    fn request(kind: BatchActionKind) -> BatchActionRequest {
        BatchActionRequest {
            kind: kind.into(),
            device_ids: vec!["serial".to_string()],
            ..BatchActionRequest::default()
        }
    }

    #[test]
    fn registers_a_handler_that_describes_each_action() {
        let kinds = all_action_kinds();

        assert!(kinds.len() > 1);
        for kind in kinds {
            assert_eq!(
                spec_for(kind).kind,
                kind,
                "{kind:?} is registered to another handler"
            );
        }
    }

    #[test]
    fn gives_every_action_a_distinct_task_slug() {
        let kinds = all_action_kinds();
        let slugs = kinds
            .iter()
            .map(|kind| spec_for(*kind).slug)
            .collect::<HashSet<_>>();

        assert_eq!(slugs.len(), kinds.len());
        assert!(slugs.iter().all(|slug| !slug.is_empty()));
    }

    #[test]
    fn gives_every_action_a_task_title() {
        assert!(
            all_action_kinds()
                .iter()
                .all(|kind| !spec_for(*kind).title_key.is_empty())
        );
    }

    #[test]
    fn reports_the_missing_field_for_each_requirement() {
        for kind in all_action_kinds() {
            let spec = spec_for(kind);
            let unmet = spec.requirement.unmet(&request(kind));

            assert_eq!(
                unmet.is_some(),
                spec.requirement != ActionRequirement::Nothing,
                "{kind:?} requirement did not match its empty request"
            );
        }
    }

    #[test]
    fn accepts_a_request_once_the_required_field_is_filled() {
        let mut filled = request(BatchActionKind::LaunchApp);
        filled.package_name = "com.example.app".to_string();

        assert!(
            spec_for(BatchActionKind::LaunchApp)
                .requirement
                .unmet(&filled)
                .is_none()
        );
    }

    #[test]
    fn fills_the_field_each_action_requires_from_a_step_value() {
        for kind in all_action_kinds() {
            let mut filled = request(BatchActionKind::Unspecified);
            fill_request(&mut filled, kind, "value");

            assert_eq!(filled.kind, kind);
            assert!(
                spec_for(kind).requirement.unmet(&filled).is_none(),
                "{kind:?} was not filled from a step value"
            );
        }

        let mut script = request(BatchActionKind::Unspecified);
        fill_request(&mut script, BatchActionKind::RunScript, "script-1");
        assert_eq!(script.script_id, "script-1");
        assert!(script.text.is_empty());
    }
}
