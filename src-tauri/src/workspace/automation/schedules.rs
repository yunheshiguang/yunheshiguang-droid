use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use chrono::{DateTime, Local};
use croner::Cron;
use croner::parser::{CronParser, Seconds, Year};
use tokio::sync::Notify;
use yunheshiguang_sdk::documents::{Collection, Document, DocumentId, DocumentStore};

use crate::workspace::clock;
use crate::workspace::contract::{RunStatus, Schedule, ScheduleState, Schedules, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::history::RunHistory;
use super::options::check_options;
use super::targets::check_target;

pub const SCHEDULES_COLLECTION: &str = "schedules";
pub const MAX_SCHEDULE_NAME_CHARS: usize = 32;
pub const MAX_SCHEDULES: usize = 100;
const MISSED_AFTER: chrono::TimeDelta = chrono::TimeDelta::minutes(2);

fn collection() -> Collection {
    Collection::parse(SCHEDULES_COLLECTION).expect("collection name is valid")
}

fn storage_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        SCHEDULE_STORAGE_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

pub fn parse_cron(expression: &str) -> Result<Cron, UiText> {
    CronParser::builder()
        .seconds(Seconds::Disallowed)
        .year(Year::Disallowed)
        .build()
        .parse(expression)
        .map_err(|error| {
            ui_text_with_params(
                SCHEDULE_CRON_INVALID_KEY,
                "",
                [("message", error.to_string())],
            )
        })
}

fn next_after(cron: &str, after: &DateTime<Local>) -> Result<DateTime<Local>, UiText> {
    parse_cron(cron)?
        .find_next_occurrence(after, false)
        .map_err(|error| {
            ui_text_with_params(
                SCHEDULE_CRON_INVALID_KEY,
                "",
                [("message", error.to_string())],
            )
        })
}

fn valid_id(id: &str) -> bool {
    id.len() <= 128
        && id
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

pub fn check_schedule(mut schedule: Schedule) -> Result<Schedule, UiText> {
    schedule.name = schedule.name.trim().to_string();
    schedule.cron = schedule.cron.trim().to_string();
    schedule.flow_id = schedule.flow_id.trim().to_string();

    if !valid_id(&schedule.id) {
        return Err(ui_text(SCHEDULE_ID_INVALID_KEY));
    }
    if schedule.name.is_empty() {
        return Err(ui_text(SCHEDULE_NAME_REQUIRED_KEY));
    }
    if schedule.name.chars().count() > MAX_SCHEDULE_NAME_CHARS {
        return Err(ui_text_with_params(
            SCHEDULE_NAME_TOO_LONG_KEY,
            "",
            [("max", MAX_SCHEDULE_NAME_CHARS.to_string())],
        ));
    }
    if schedule.flow_id.is_empty() {
        return Err(ui_text(SCHEDULE_FLOW_REQUIRED_KEY));
    }
    check_target(schedule.target.as_option())?;
    check_options(schedule.options.as_option())?;
    parse_cron(&schedule.cron)?;

    Ok(schedule)
}

struct Entry {
    schedule: Schedule,
    next: Option<DateTime<Local>>,
    problem: Option<UiText>,
}

impl Entry {
    fn new(schedule: Schedule, now: &DateTime<Local>) -> Self {
        let (next, problem) = if schedule.enabled {
            match next_after(&schedule.cron, now) {
                Ok(next) => (Some(next), None),
                Err(problem) => (None, Some(problem)),
            }
        } else {
            (None, None)
        };

        Self {
            schedule,
            next,
            problem,
        }
    }
}

pub struct ScheduleBook {
    store: Option<Arc<dyn DocumentStore>>,
    entries: Mutex<Vec<Entry>>,
    changed: Notify,
}

impl ScheduleBook {
    fn with(store: Option<Arc<dyn DocumentStore>>, schedules: Vec<Schedule>) -> Self {
        let now = Local::now();

        Self {
            store,
            entries: Mutex::new(
                schedules
                    .into_iter()
                    .map(|schedule| Entry::new(schedule, &now))
                    .collect(),
            ),
            changed: Notify::new(),
        }
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        Self::with(None, Vec::new())
    }

    pub fn open(store: Arc<dyn DocumentStore>) -> Self {
        let schedules = match store.list(&collection()) {
            Ok(documents) => documents
                .into_iter()
                .filter_map(|document| {
                    let id = document.id.as_str().to_string();

                    yunheshiguang_contract::json::from_str::<Schedule>(&document.body)
                        .inspect_err(|error| {
                            log::warn!("automation: schedule {id} is unreadable: {error}");
                        })
                        .ok()
                        .map(|schedule| Schedule { id, ..schedule })
                })
                .collect(),
            Err(error) => {
                log::warn!("automation: schedules are unreadable: {error}");
                Vec::new()
            }
        };

        Self::with(Some(store), schedules)
    }

    fn entries(&self) -> MutexGuard<'_, Vec<Entry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self, schedule: &Schedule) -> Result<(), UiText> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let body = serde_json::to_string(schedule).map_err(storage_failed)?;
        let document = DocumentId::parse(&schedule.id)
            .and_then(|id| Document::new(id, body))
            .map_err(storage_failed)?;

        store.save(&collection(), &document).map_err(storage_failed)
    }

    fn delete(&self, id: &str) -> Result<(), UiText> {
        let Some(store) = &self.store else {
            return Ok(());
        };

        DocumentId::parse(id)
            .and_then(|id| store.remove(&collection(), &id))
            .map_err(storage_failed)
    }

    pub async fn changed(&self) {
        self.changed.notified().await;
    }

    pub fn states(&self, history: &RunHistory) -> Schedules {
        let mut schedules = self
            .entries()
            .iter()
            .map(|entry| {
                let last = history.latest_for_schedule(&entry.schedule.id);

                ScheduleState {
                    schedule: entry.schedule.clone().into(),
                    next_run_at: entry.next.as_ref().map(clock::from_datetime).into(),
                    last_run_id: last.as_ref().map(|run| run.id.clone()).unwrap_or_default(),
                    last_status: last
                        .as_ref()
                        .map_or(RunStatus::Unspecified.into(), |run| run.status),
                    last_run_at: (last.and_then(|run| run.started_at.into_option())).into(),
                    problem: entry.problem.clone().into(),
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();
        schedules.sort_by(|left, right| {
            let name = |state: &ScheduleState| {
                state
                    .schedule
                    .as_option()
                    .map(|schedule| (schedule.name.clone(), schedule.id.clone()))
            };
            name(left).cmp(&name(right))
        });

        Schedules {
            schedules,
            ..Default::default()
        }
    }

    pub fn save(&self, schedule: Schedule) -> Result<(), UiText> {
        let mut entries = self.entries();
        let position = entries
            .iter()
            .position(|entry| entry.schedule.id == schedule.id);

        if position.is_none() && entries.len() >= MAX_SCHEDULES {
            return Err(ui_text_with_params(
                SCHEDULE_LIMIT_KEY,
                "",
                [("max", MAX_SCHEDULES.to_string())],
            ));
        }
        self.write(&schedule)?;

        let entry = Entry::new(schedule, &Local::now());
        match position {
            Some(index) => entries[index] = entry,
            None => entries.push(entry),
        }
        drop(entries);
        self.changed.notify_one();
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<Option<Schedule>, UiText> {
        let mut entries = self.entries();
        let Some(index) = entries.iter().position(|entry| entry.schedule.id == id) else {
            return Ok(None);
        };
        self.delete(id)?;
        let removed = entries.remove(index);
        drop(entries);
        self.changed.notify_one();
        Ok(Some(removed.schedule))
    }

    pub fn due(&self, now: &DateTime<Local>) -> Vec<Schedule> {
        let mut due = Vec::new();

        for entry in self.entries().iter_mut() {
            let Some(next) = entry.next.filter(|next| *next <= *now) else {
                continue;
            };
            if *now - next <= MISSED_AFTER {
                due.push(entry.schedule.clone());
            } else {
                log::info!(
                    "automation: schedule {} missed {next}; waiting for the next time",
                    entry.schedule.id
                );
            }
            entry.next = next_after(&entry.schedule.cron, now).ok();
        }
        due
    }

    pub fn next_due(&self) -> Option<DateTime<Local>> {
        self.entries().iter().filter_map(|entry| entry.next).min()
    }

    pub fn record_problem(&self, id: &str, problem: Option<UiText>) {
        if let Some(entry) = self
            .entries()
            .iter_mut()
            .find(|entry| entry.schedule.id == id)
        {
            entry.problem = problem;
        }
    }
}
