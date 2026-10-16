use std::collections::BTreeMap;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use chrono::{DateTime, SecondsFormat};
use yunheshiguang_sdk::fs::open_atomic;

use crate::workspace::contract::{
    AuditEntry, AuditExportFormat, AuditOutcome, AuditQuery, GovernedAction, UiText,
};
use crate::workspace::governance::Governance;
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

use super::query::MAX_PAGE_SIZE;
use super::sqlite::{RETAINED_ENTRIES, storage_failed};

const MAX_EXPORTED: u32 = RETAINED_ENTRIES as u32;

const CSV_HEADER: [&str; 12] = [
    "id",
    "occurred_at",
    "action",
    "outcome",
    "actor",
    "operator_id",
    "operator_name",
    "operator_roles",
    "targets",
    "groups",
    "detail_key",
    "detail_params",
];

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

fn neutralize(cell: String) -> String {
    if cell.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{cell}")
    } else {
        cell
    }
}

fn occurred_at(entry: &AuditEntry) -> String {
    entry
        .occurred_at
        .as_option()
        .and_then(|at| DateTime::from_timestamp(at.seconds, u32::try_from(at.nanos).ok()?))
        .map(|at| at.to_rfc3339_opts(SecondsFormat::Millis, true))
        .unwrap_or_default()
}

fn csv_row(entry: &AuditEntry) -> [String; 12] {
    let operator = entry.operator.clone().into_option().unwrap_or_default();
    let detail = entry.detail.clone().into_option().unwrap_or_default();
    let params = detail.params.iter().collect::<BTreeMap<_, _>>();

    [
        entry.id.clone(),
        occurred_at(entry),
        GovernedAction::known(entry.action)
            .unwrap_or_default()
            .proto_name()
            .to_string(),
        AuditOutcome::known(entry.outcome)
            .unwrap_or_default()
            .proto_name()
            .to_string(),
        entry.actor.clone(),
        operator.id,
        operator.name,
        operator.roles.join(" "),
        entry.targets.join(" "),
        entry.groups.join(" "),
        detail.key,
        if params.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&params).unwrap_or_default()
        },
    ]
    .map(neutralize)
}

enum Rows<W: Write> {
    Csv(Box<csv::Writer<W>>),
    JsonLines(W),
}

impl<W: Write> Rows<W> {
    fn start(format: AuditExportFormat, mut inner: W) -> io::Result<Self> {
        match format {
            AuditExportFormat::Jsonl => Ok(Self::JsonLines(inner)),
            _ => {
                inner.write_all(UTF8_BOM)?;
                let mut writer = csv::Writer::from_writer(inner);
                writer.write_record(CSV_HEADER)?;
                Ok(Self::Csv(Box::new(writer)))
            }
        }
    }

    fn write(&mut self, entry: &AuditEntry) -> io::Result<()> {
        match self {
            Self::Csv(writer) => Ok(writer.write_record(csv_row(entry))?),
            Self::JsonLines(writer) => {
                serde_json::to_writer(&mut *writer, entry)?;
                writer.write_all(b"\n")
            }
        }
    }

    fn finish(self) -> io::Result<()> {
        match self {
            Self::Csv(mut writer) => writer.flush(),
            Self::JsonLines(mut writer) => writer.flush(),
        }
    }
}

pub fn export(
    governance: &Governance,
    query: AuditQuery,
    format: AuditExportFormat,
    path: &Path,
) -> Result<u32, UiText> {
    if !path.parent().is_some_and(Path::is_dir) {
        return Err(ui_text(AUDIT_EXPORT_PATH_KEY));
    }
    let file = open_atomic(path).map_err(storage_failed)?;
    let mut rows = Rows::start(format, BufWriter::new(&file)).map_err(storage_failed)?;
    let mut page_query = AuditQuery {
        limit: MAX_PAGE_SIZE,
        before_id: String::new(),
        ..query
    };
    let mut exported = 0_u32;

    loop {
        let page = governance.query_audit(&page_query);
        if let Some(problem) = page.problem.into_option() {
            return Err(problem);
        }

        for entry in &page.entries {
            rows.write(entry).map_err(storage_failed)?;
            exported += 1;
        }

        if page.next_before_id.is_empty() || exported >= MAX_EXPORTED {
            break;
        }
        page_query.before_id = page.next_before_id;
    }

    rows.finish().map_err(storage_failed)?;
    file.commit().map_err(storage_failed)?;
    Ok(exported)
}
