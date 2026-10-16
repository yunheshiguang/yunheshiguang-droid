use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};
use yunheshiguang_contract::EnumValue;
pub use yunheshiguang_sdk::governance::timestamp_millis;

use crate::workspace::contract::{AuditEntry, AuditPage, AuditQuery};

pub const DEFAULT_PAGE_SIZE: u32 = 100;
pub const MAX_PAGE_SIZE: u32 = 500;

pub fn page_size(limit: u32) -> u32 {
    if limit == 0 {
        DEFAULT_PAGE_SIZE
    } else {
        limit.min(MAX_PAGE_SIZE)
    }
}

fn conditions(query: &AuditQuery) -> (String, Vec<Value>) {
    let mut clauses = Vec::new();
    let mut values = Vec::new();
    let mut any_of = |column: &str, codes: Vec<i32>| {
        if !codes.is_empty() {
            clauses.push(format!(
                "{column} IN ({})",
                vec!["?"; codes.len()].join(", ")
            ));
            values.extend(codes.iter().map(|code| Value::Integer(i64::from(*code))));
        }
    };

    any_of(
        "action",
        query.actions.iter().map(EnumValue::to_i32).collect(),
    );
    any_of(
        "outcome",
        query.outcomes.iter().map(EnumValue::to_i32).collect(),
    );

    let operator_id = query.operator_id.trim();
    if !operator_id.is_empty() {
        clauses.push("operator_id = ?".to_string());
        values.push(Value::Text(operator_id.to_string()));
    }
    let actor = query.actor.trim();
    if !actor.is_empty() {
        let channel = format!("{actor}:");
        clauses.push("(actor = ? OR substr(actor, 1, ?) = ?)".to_string());
        values.push(Value::Text(actor.to_string()));
        values.push(Value::Integer(
            i64::try_from(channel.chars().count()).unwrap_or(i64::MAX),
        ));
        values.push(Value::Text(channel));
    }

    if let Some(since) = query.since.as_option() {
        clauses.push("occurred_at >= ?".to_string());
        values.push(Value::Integer(timestamp_millis(since)));
    }
    if let Some(until) = query.until.as_option() {
        clauses.push("occurred_at < ?".to_string());
        values.push(Value::Integer(timestamp_millis(until)));
    }
    if !query.before_id.trim().is_empty() {
        clauses.push("seq < (SELECT seq FROM entries WHERE id = ?)".to_string());
        values.push(Value::Text(query.before_id.trim().to_string()));
    }

    let condition = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };

    (condition, values)
}

pub fn read_page(connection: &Connection, query: &AuditQuery) -> rusqlite::Result<AuditPage> {
    let size = page_size(query.limit) as usize;
    let (condition, mut values) = conditions(query);
    values.push(Value::Integer(i64::try_from(size + 1).unwrap_or(i64::MAX)));

    let mut statement = connection.prepare(&format!(
        "SELECT id, entry FROM entries {condition} ORDER BY seq DESC LIMIT ?"
    ))?;
    let rows = statement
        .query_map(params_from_iter(values), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let next_before_id = rows
        .get(size)
        .and(rows.get(size - 1))
        .map(|(id, _)| id.clone())
        .unwrap_or_default();
    let entries = rows
        .into_iter()
        .take(size)
        .filter_map(|(id, body)| {
            yunheshiguang_contract::json::from_str::<AuditEntry>(&body)
                .inspect_err(|error| log::warn!("audit store: entry {id} is unreadable: {error}"))
                .ok()
        })
        .collect();

    Ok(AuditPage {
        entries,
        next_before_id,
        problem: Default::default(),
        ..Default::default()
    })
}
