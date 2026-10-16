use std::path::PathBuf;
use std::sync::Arc;
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::{
    AuditExportFormat, AuditOutcome, AuditPage, AuditQuery, AuditReport, ExportAuditRequest,
    ExportAuditResult, GovernedAction, ReportAuditRequest, UiText,
};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::export::export;
use super::report::report;
use super::sqlite::storage_failed;

fn refused(detail: UiText) -> ExportAuditResult {
    ExportAuditResult {
        accepted: false,
        detail: detail.into(),
        exported: 0,
        ..Default::default()
    }
}

pub async fn export_as(
    governance: &Arc<Governance>,
    request: ExportAuditRequest,
    principal: &Principal,
) -> ExportAuditResult {
    let format = AuditExportFormat::known(request.format).unwrap_or_default();
    let path = PathBuf::from(request.path.trim());

    if format == AuditExportFormat::Unspecified {
        return refused(ui_text(AUDIT_EXPORT_FORMAT_KEY));
    }
    if !path.is_absolute() {
        return refused(ui_text(AUDIT_EXPORT_PATH_KEY));
    }
    if let Err(reason) = governance.authorize(GovernedAction::ViewAudit, principal, &[]) {
        return refused(reason);
    }

    let ticket = match governance.begin(GovernedAction::ExportAudit, principal, &[]) {
        Ok(ticket) => ticket,
        Err(reason) => return refused(reason),
    };
    let worker = Arc::clone(governance);
    let query = request.query.into_option().unwrap_or_default();
    let destination = path.clone();
    let outcome = run_blocking(move || Ok(export(&worker, query, format, &destination)))
        .await
        .unwrap_or_else(|error| Err(storage_failed(error)));

    match outcome {
        Ok(exported) => {
            ticket.finish(
                AuditOutcome::Succeeded,
                Some(ui_text_with_params(
                    AUDIT_AUDIT_EXPORTED_KEY,
                    "",
                    [
                        ("count", exported.to_string()),
                        ("format", format.proto_name().to_string()),
                    ],
                )),
            );
            ExportAuditResult {
                accepted: true,
                detail: ui_text_with_params(
                    AUDIT_EXPORTED_KEY,
                    "",
                    [
                        ("count", exported.to_string()),
                        ("path", path.display().to_string()),
                    ],
                )
                .into(),
                exported,
                ..Default::default()
            }
        }
        Err(detail) => {
            log::warn!("audit export failed: {}", detail.key);
            ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
            refused(detail)
        }
    }
}

pub async fn query_as(
    governance: &Arc<Governance>,
    query: AuditQuery,
    principal: &Principal,
) -> Result<AuditPage, String> {
    if let Err(reason) = governance.authorize(GovernedAction::ViewAudit, principal, &[]) {
        return Ok(AuditPage {
            problem: reason.into(),
            ..AuditPage::default()
        });
    }
    let governance = Arc::clone(governance);

    run_blocking(move || Ok(governance.query_audit(&query))).await
}

pub async fn report_as(
    governance: &Arc<Governance>,
    request: ReportAuditRequest,
    principal: &Principal,
) -> Result<AuditReport, String> {
    if let Err(reason) = governance.authorize(GovernedAction::ViewAudit, principal, &[]) {
        return Ok(AuditReport {
            problem: reason.into(),
            ..AuditReport::default()
        });
    }
    let governance = Arc::clone(governance);

    run_blocking(move || {
        Ok(report(
            &governance,
            request.query.into_option().unwrap_or_default(),
            request.utc_offset_minutes,
        ))
    })
    .await
}
