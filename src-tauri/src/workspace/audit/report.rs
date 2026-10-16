use std::collections::{BTreeSet, HashMap};
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use chrono::{DateTime, FixedOffset};

use crate::workspace::contract::{
    AuditEntry, AuditOutcome, AuditQuery, AuditReport, AuditTally, GovernedAction,
};
use crate::workspace::governance::Governance;

use super::query::MAX_PAGE_SIZE;
use super::sqlite::RETAINED_ENTRIES;

const MAX_COUNTED: u32 = RETAINED_ENTRIES as u32;
const MAX_OFFSET_MINUTES: i32 = 14 * 60;

#[derive(Default)]
struct Counts {
    label: String,
    total: u32,
    succeeded: u32,
    failed: u32,
    denied: u32,
}

impl Counts {
    fn add(&mut self, outcome: i32) {
        self.total += 1;
        match AuditOutcome::known(outcome) {
            Some(AuditOutcome::Succeeded) => self.succeeded += 1,
            Some(AuditOutcome::Failed) => self.failed += 1,
            Some(AuditOutcome::Denied) => self.denied += 1,
            _ => {}
        }
    }
}

#[derive(Default)]
struct Tallies {
    operators: HashMap<String, Counts>,
    groups: HashMap<String, Counts>,
    actions: HashMap<String, Counts>,
    days: HashMap<String, Counts>,
}

fn rows(counts: HashMap<String, Counts>) -> Vec<AuditTally> {
    counts
        .into_iter()
        .map(|(key, counts)| AuditTally {
            key,
            label: counts.label,
            total: counts.total,
            succeeded: counts.succeeded,
            failed: counts.failed,
            denied: counts.denied,
            ..Default::default()
        })
        .collect()
}

fn busiest_first(counts: HashMap<String, Counts>) -> Vec<AuditTally> {
    let mut rows = rows(counts);
    rows.sort_by(|left, right| right.total.cmp(&left.total).then(left.key.cmp(&right.key)));
    rows
}

impl Tallies {
    fn count(&mut self, entry: &AuditEntry, offset: FixedOffset) {
        let operator = entry.operator.clone().into_option().unwrap_or_default();
        let counted = self.operators.entry(operator.id).or_default();
        counted.label = operator.name;
        counted.add(entry.outcome.to_i32());

        for group in entry.groups.iter().collect::<BTreeSet<_>>() {
            self.groups
                .entry(group.clone())
                .or_default()
                .add(entry.outcome.to_i32());
        }

        let action = GovernedAction::known(entry.action)
            .unwrap_or_default()
            .proto_name()
            .to_string();
        self.actions
            .entry(action)
            .or_default()
            .add(entry.outcome.to_i32());

        if let Some(day) = entry
            .occurred_at
            .as_option()
            .and_then(|at| DateTime::from_timestamp(at.seconds, 0))
        {
            let day = day.with_timezone(&offset).format("%Y-%m-%d").to_string();
            self.days
                .entry(day)
                .or_default()
                .add(entry.outcome.to_i32());
        }
    }

    fn into_report(self, entries: u32, truncated: bool) -> AuditReport {
        let mut by_day = rows(self.days);
        by_day.sort_by(|left, right| left.key.cmp(&right.key));

        AuditReport {
            by_operator: busiest_first(self.operators),
            by_group: busiest_first(self.groups),
            by_action: busiest_first(self.actions),
            by_day,
            entries,
            truncated,
            problem: Default::default(),
            ..Default::default()
        }
    }
}

pub fn report(governance: &Governance, query: AuditQuery, utc_offset_minutes: i32) -> AuditReport {
    let offset = FixedOffset::east_opt(
        utc_offset_minutes.clamp(-MAX_OFFSET_MINUTES, MAX_OFFSET_MINUTES) * 60,
    )
    .unwrap_or_else(|| FixedOffset::east_opt(0).expect("UTC is a valid offset"));
    let mut page_query = AuditQuery {
        limit: MAX_PAGE_SIZE,
        before_id: String::new(),
        ..query
    };
    let mut tallies = Tallies::default();
    let mut counted = 0_u32;

    loop {
        let page = governance.query_audit(&page_query);
        if let Some(problem) = page.problem.into_option() {
            return AuditReport {
                problem: problem.into(),
                ..AuditReport::default()
            };
        }

        for entry in &page.entries {
            tallies.count(entry, offset);
            counted += 1;
        }

        if page.next_before_id.is_empty() {
            return tallies.into_report(counted, false);
        }
        if counted >= MAX_COUNTED {
            return tallies.into_report(counted, true);
        }
        page_query.before_id = page.next_before_id;
    }
}

#[cfg(test)]
mod tests {
    use yunheshiguang_contract::wkt::Timestamp;

    use super::*;
    use crate::workspace::contract::Operator;

    fn entry(operator: &str, groups: &[&str], outcome: AuditOutcome, seconds: i64) -> AuditEntry {
        AuditEntry {
            occurred_at: Timestamp {
                seconds,
                nanos: 0,
                ..Default::default()
            }
            .into(),
            action: GovernedAction::RunBatchAction.into(),
            outcome: outcome.into(),
            operator: Operator {
                id: format!("local:{operator}"),
                name: operator.to_string(),
                ..Operator::default()
            }
            .into(),
            groups: groups.iter().map(ToString::to_string).collect(),
            ..AuditEntry::default()
        }
    }

    #[test]
    fn counts_each_operator_group_action_and_local_day() {
        let utc8 = FixedOffset::east_opt(8 * 3600).unwrap();
        let mut tallies = Tallies::default();
        for entry in [
            entry(
                "alice",
                &["shelf-a", "shelf-a"],
                AuditOutcome::Succeeded,
                1_791_304_200,
            ),
            entry("alice", &["shelf-b"], AuditOutcome::Failed, 1_791_217_800),
            entry("bob", &["ungrouped"], AuditOutcome::Denied, 1_791_217_800),
        ] {
            tallies.count(&entry, utc8);
        }

        let report = tallies.into_report(3, false);

        assert_eq!(report.by_operator[0].key, "local:alice");
        assert_eq!(report.by_operator[0].label, "alice");
        assert_eq!(
            (report.by_operator[0].total, report.by_operator[0].failed),
            (2, 1)
        );
        assert_eq!(report.by_operator[1].denied, 1);
        let shelf_a = report
            .by_group
            .iter()
            .find(|row| row.key == "shelf-a")
            .unwrap();
        assert_eq!(shelf_a.total, 1, "two devices in one group count once");
        assert_eq!(report.by_action[0].key, "GOVERNED_ACTION_RUN_BATCH_ACTION");
        assert_eq!(report.by_action[0].total, 3);
        let days = report
            .by_day
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(days, ["2026-10-06", "2026-10-07"]);
    }
}
