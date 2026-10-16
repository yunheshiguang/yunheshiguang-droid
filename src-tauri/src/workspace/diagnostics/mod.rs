mod spec;

use spec::spec_for;

use crate::workspace::contract::{ConnectionHealth, DiagnosticCode, DiagnosticSignal, UiText};
use crate::workspace::text::ui_text;

pub fn signal(id: impl Into<String>, code: DiagnosticCode, impact: UiText) -> DiagnosticSignal {
    DiagnosticSignal {
        id: id.into(),
        code: code.into(),
        health: health_for(code).into(),
        title: title_text(code).into(),
        impact: impact.into(),
        recovery: recovery_text(code).into(),
        ..Default::default()
    }
}

pub fn health_for(code: DiagnosticCode) -> ConnectionHealth {
    spec_for(code).health
}

pub fn title_key(code: DiagnosticCode) -> &'static str {
    spec_for(code).title_key
}

pub fn recovery_key(code: DiagnosticCode) -> &'static str {
    spec_for(code).recovery_key
}

pub fn title_text(code: DiagnosticCode) -> UiText {
    ui_text(title_key(code))
}

pub fn recovery_text(code: DiagnosticCode) -> UiText {
    ui_text(recovery_key(code))
}

#[cfg(test)]
mod tests;
