use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, Transaction, params};
use rusqlite_migration::{M, Migrations};
use yunheshiguang_sdk::governance::{AuditStore, provider_info as provider};

use crate::workspace::contract::{AuditEntry, AuditPage, AuditQuery, GovernanceProvider, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text_with_params;

use super::query::{read_page, timestamp_millis};

pub const RETAINED_ENTRIES: i64 = 100_000;
const PRUNE_EVERY: usize = 1_000;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const FLUSH_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_millis(100),
    Duration::from_millis(500),
    Duration::from_secs(2),
];
const MAX_UNWRITTEN: usize = 10_000;

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(
        "CREATE TABLE entries (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL UNIQUE,
            occurred_at INTEGER NOT NULL,
            action INTEGER NOT NULL,
            outcome INTEGER NOT NULL,
            actor TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            entry TEXT NOT NULL
        ) STRICT;
        CREATE INDEX entries_occurred_at ON entries (occurred_at);
        CREATE INDEX entries_operator ON entries (operator_id);",
    )])
}

enum Job {
    Append(Box<AuditEntry>),
    Flush(SyncSender<()>),
}

pub struct SqliteAuditStore {
    jobs: Mutex<Sender<Job>>,
    reader: Mutex<Connection>,
}

pub fn storage_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        AUDIT_STORAGE_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn open_connection(path: &Path, flags: OpenFlags) -> rusqlite::Result<Connection> {
    let connection = Connection::open_with_flags(path, flags)?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    Ok(connection)
}

impl SqliteAuditStore {
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_retaining(path, RETAINED_ENTRIES)
    }

    pub fn open_retaining(path: &Path, retained: i64) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)?;
        }

        let mut writer = open_connection(path, OpenFlags::default())?;
        writer
            .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
        writer.pragma_update(None, "synchronous", "NORMAL")?;
        migrations().to_latest(&mut writer)?;
        prune(&writer, retained)?;

        let reader = open_connection(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let (jobs, queue) = mpsc::channel();
        thread::Builder::new()
            .name("audit-writer".to_string())
            .spawn(move || write_queue(writer, &queue, retained))?;

        Ok(Self {
            jobs: Mutex::new(jobs),
            reader: Mutex::new(reader),
        })
    }

    fn send(&self, job: Job) -> bool {
        self.jobs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .send(job)
            .is_ok()
    }
}

impl AuditStore for SqliteAuditStore {
    fn describe(&self) -> GovernanceProvider {
        provider(
            "core.sqlite-audit",
            GOVERNANCE_SQLITE_AUDIT_LABEL_KEY,
            GOVERNANCE_SQLITE_AUDIT_DESCRIPTION_KEY,
        )
    }

    fn append(&self, entry: &AuditEntry) {
        if !self.send(Job::Append(Box::new(entry.clone()))) {
            log::error!(
                "audit store: the writer has stopped, entry {} is lost",
                entry.id
            );
        }
    }

    fn query(&self, query: &AuditQuery) -> Result<AuditPage, UiText> {
        self.flush();

        let reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
        read_page(&reader, query).map_err(storage_failed)
    }

    fn flush(&self) {
        let (done, wait) = mpsc::sync_channel(1);

        if self.send(Job::Flush(done)) && wait.recv_timeout(FLUSH_TIMEOUT).is_err() {
            log::warn!("audit store: the writer did not catch up in time");
        }
    }
}

fn write_with_retries(
    connection: &mut Connection,
    entries: &[Box<AuditEntry>],
) -> rusqlite::Result<()> {
    let mut outcome = connection
        .transaction()
        .and_then(|transaction| insert_all(transaction, entries));

    for delay in RETRY_DELAYS {
        if outcome.is_ok() {
            break;
        }
        thread::sleep(delay);
        outcome = connection
            .transaction()
            .and_then(|transaction| insert_all(transaction, entries));
    }
    outcome
}

fn write_queue(mut connection: Connection, queue: &Receiver<Job>, retained: i64) {
    let mut since_prune = 0;
    let mut unwritten: Vec<Box<AuditEntry>> = Vec::new();

    while let Ok(first) = queue.recv() {
        let mut flushed = Vec::new();

        for job in std::iter::once(first).chain(queue.try_iter()) {
            match job {
                Job::Append(entry) => unwritten.push(entry),
                Job::Flush(done) => flushed.push(done),
            }
        }

        if !unwritten.is_empty() {
            match write_with_retries(&mut connection, &unwritten) {
                Ok(()) => {
                    since_prune += unwritten.len();
                    unwritten.clear();
                }
                Err(error) => {
                    log::error!(
                        "audit store: {} entries are waiting to be written: {error}",
                        unwritten.len()
                    );
                    if unwritten.len() > MAX_UNWRITTEN {
                        let dropped = unwritten.len() - MAX_UNWRITTEN;
                        unwritten.drain(..dropped);
                        log::error!("audit store: dropped the {dropped} oldest unwritten entries");
                    }
                }
            }
        }

        if since_prune >= PRUNE_EVERY {
            since_prune = 0;
            if let Err(error) = prune(&connection, retained) {
                log::warn!("audit store: pruning failed: {error}");
            }
        }

        for done in flushed {
            let _ = done.send(());
        }
    }
}

fn insert_all(transaction: Transaction<'_>, entries: &[Box<AuditEntry>]) -> rusqlite::Result<()> {
    {
        let mut insert = transaction.prepare_cached(
            "INSERT OR IGNORE INTO entries
                (id, occurred_at, action, outcome, actor, operator_id, entry)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;

        for entry in entries {
            let body = serde_json::to_string(entry.as_ref())
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;

            insert.execute(params![
                entry.id,
                entry.occurred_at.as_option().map_or(0, timestamp_millis),
                entry.action.to_i32(),
                entry.outcome.to_i32(),
                entry.actor,
                entry
                    .operator
                    .as_option()
                    .map(|operator| operator.id.as_str())
                    .unwrap_or_default(),
                body,
            ])?;
        }
    }

    transaction.commit()
}

fn prune(connection: &Connection, retained: i64) -> rusqlite::Result<usize> {
    connection.execute(
        "DELETE FROM entries WHERE seq <= (SELECT MAX(seq) FROM entries) - ?1",
        [retained],
    )
}
