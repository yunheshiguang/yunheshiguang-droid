use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::{AuditEntry, GovernanceProvider};
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::UiText;
use yunheshiguang_sdk::governance::{
    AuditPage, AuditQuery, AuditStore, audit_matches, provider_info, timestamp_millis,
};
use yunheshiguang_sdk::text::ui_text_with_params;

use crate::messages::*;

const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_READ_BYTES: u64 = 8 * MAX_FILE_BYTES;
const RETRY_AFTER: Duration = Duration::from_secs(5);
const MAX_PENDING_LINES: usize = 10_000;
const DEFAULT_PAGE_SIZE: usize = 100;
const MAX_PAGE_SIZE: usize = 500;
const FLUSH_TIMEOUT: Duration = Duration::from_secs(5);

enum Job {
    Append(String),
    Flush(SyncSender<()>),
}

type FileStamp = (u64, Option<SystemTime>);
type Entries = Arc<Vec<AuditEntry>>;
type Stamps = Vec<(PathBuf, FileStamp)>;

#[derive(Default)]
struct ReadCache {
    files: HashMap<PathBuf, (FileStamp, Entries)>,
    merged: Option<(Stamps, Entries)>,
}

pub struct TeamAuditStore {
    folder: PathBuf,
    jobs: Mutex<Sender<Job>>,
    cache: Mutex<ReadCache>,
}

fn ends_with_newline(file: &mut fs::File) -> io::Result<bool> {
    if file.metadata()?.len() == 0 {
        return Ok(true);
    }
    let mut last = [0_u8; 1];
    file.seek(SeekFrom::End(-1))?;
    file.read_exact(&mut last)?;
    Ok(last[0] == b'\n')
}

fn parse_lines(bytes: &[u8]) -> Vec<AuditEntry> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter_map(|line| yunheshiguang_sdk::contract::json::from_slice::<AuditEntry>(line).ok())
        .collect()
}

fn rotated(path: &Path) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();

    path.with_extension(format!("{millis}.jsonl"))
}

fn write_lines(path: &Path, lines: &[String]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if fs::metadata(path).is_ok_and(|metadata| metadata.len() > MAX_FILE_BYTES)
        && let Err(error) = fs::rename(path, rotated(path))
    {
        log::warn!("team audit: could not rotate {}: {error}", path.display());
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path)?;
    let mut buffer = Vec::new();
    if !ends_with_newline(&mut file)? {
        buffer.push(b'\n');
    }
    for line in lines {
        buffer.extend_from_slice(line.as_bytes());
        buffer.push(b'\n');
    }
    file.write_all(&buffer)?;
    file.flush()
}

fn next_job(queue: &Receiver<Job>, waiting: bool) -> Result<Option<Job>, RecvTimeoutError> {
    if waiting {
        match queue.recv_timeout(RETRY_AFTER) {
            Ok(job) => Ok(Some(job)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(closed) => Err(closed),
        }
    } else {
        queue
            .recv()
            .map(Some)
            .map_err(|_| RecvTimeoutError::Disconnected)
    }
}

fn write_queue(path: &Path, queue: &Receiver<Job>) {
    let mut pending = Vec::new();
    let mut open = true;

    while open || !pending.is_empty() {
        let first = if open {
            next_job(queue, !pending.is_empty()).unwrap_or_else(|_| {
                open = false;
                None
            })
        } else {
            None
        };
        let mut flushed = Vec::new();
        for job in first.into_iter().chain(queue.try_iter()) {
            match job {
                Job::Append(line) => pending.push(line),
                Job::Flush(done) => flushed.push(done),
            }
        }
        if !pending.is_empty() {
            match write_lines(path, &pending) {
                Ok(()) => pending.clear(),
                Err(error) => {
                    log::warn!(
                        "team audit: could not append to {}: {error}",
                        path.display()
                    );
                    let keep = if open { MAX_PENDING_LINES } else { 0 };
                    if pending.len() > keep {
                        let dropped = pending.len() - keep;
                        pending.drain(..dropped);
                        log::error!("team audit: {dropped} entries could not be written");
                    }
                }
            }
        }
        for done in flushed {
            let _ = done.send(());
        }
    }
}

fn millis(entry: &AuditEntry) -> i64 {
    entry.occurred_at.as_option().map_or(0, timestamp_millis)
}

impl TeamAuditStore {
    pub fn open(folder: PathBuf, computer: &str) -> Self {
        let path = folder.join(format!("{computer}.jsonl"));
        let (jobs, queue) = mpsc::channel();
        if let Err(error) = thread::Builder::new()
            .name("team-audit-writer".to_string())
            .spawn(move || write_queue(&path, &queue))
        {
            log::error!("team audit: the writer did not start: {error}");
        }

        Self {
            folder,
            jobs: Mutex::new(jobs),
            cache: Mutex::new(ReadCache::default()),
        }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    fn send(&self, job: Job) -> bool {
        self.jobs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .send(job)
            .is_ok()
    }

    fn stamps(&self) -> io::Result<Stamps> {
        let files = match fs::read_dir(&self.folder) {
            Ok(files) => files,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut stamps = Vec::new();

        for file in files.flatten() {
            let path = file.path();
            if path
                .extension()
                .is_none_or(|extension| extension != "jsonl")
            {
                continue;
            }
            match file.metadata() {
                Ok(metadata) if metadata.is_file() => {
                    stamps.push((path, (metadata.len(), metadata.modified().ok())));
                }
                Ok(_) => {}
                Err(error) => log::warn!("team audit {}: {error}", path.display()),
            }
        }
        stamps.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(stamps)
    }

    fn read_all(&self) -> io::Result<Entries> {
        let stamps = self.stamps()?;
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);

        if let Some((seen, merged)) = &cache.merged
            && *seen == stamps
        {
            return Ok(Arc::clone(merged));
        }

        let mut entries = Vec::new();
        let mut files = HashMap::new();
        for (path, stamp) in &stamps {
            if stamp.0 > MAX_READ_BYTES {
                log::warn!("team audit {}: too large to read", path.display());
                continue;
            }
            let parsed = match cache.files.get(path) {
                Some((cached, parsed)) if cached == stamp => Arc::clone(parsed),
                _ => match fs::read(path) {
                    Ok(bytes) => Arc::new(parse_lines(&bytes)),
                    Err(error) => {
                        log::warn!("team audit {}: {error}", path.display());
                        continue;
                    }
                },
            };
            entries.extend(parsed.iter().cloned());
            files.insert(path.clone(), (*stamp, parsed));
        }
        entries.sort_by(|left, right| {
            millis(right)
                .cmp(&millis(left))
                .then_with(|| right.id.cmp(&left.id))
        });
        entries.dedup_by(|later, earlier| later.id == earlier.id);

        let merged = Arc::new(entries);
        cache.files = files;
        cache.merged = Some((stamps, Arc::clone(&merged)));
        Ok(merged)
    }
}

fn read_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        REFERENCE_AUDIT_READ_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

impl AuditStore for TeamAuditStore {
    fn describe(&self) -> GovernanceProvider {
        provider_info(
            "reference.team-audit",
            REFERENCE_AUDIT_LABEL_KEY,
            REFERENCE_AUDIT_DESCRIPTION_KEY,
        )
    }

    fn append(&self, entry: &AuditEntry) {
        match serde_json::to_string(entry) {
            Ok(line) => {
                if !self.send(Job::Append(line)) {
                    log::error!(
                        "team audit: the writer has stopped, entry {} is lost",
                        entry.id
                    );
                }
            }
            Err(error) => log::warn!("team audit: entry {} is unwritable: {error}", entry.id),
        }
    }

    fn query(&self, query: &AuditQuery) -> Result<AuditPage, UiText> {
        self.flush();

        let size = match usize::try_from(query.limit).unwrap_or(MAX_PAGE_SIZE) {
            0 => DEFAULT_PAGE_SIZE,
            limit => limit.min(MAX_PAGE_SIZE),
        };
        let before = query.before_id.trim();
        let all = self.read_all().map_err(read_failed)?;
        let mut selected = all
            .iter()
            .filter(|entry| audit_matches(query, entry))
            .skip_while(|entry| !before.is_empty() && entry.id != before)
            .skip(usize::from(!before.is_empty()))
            .take(size + 1)
            .cloned()
            .collect::<Vec<_>>();
        let more = selected.len() > size;
        selected.truncate(size);

        Ok(AuditPage {
            next_before_id: if more {
                selected
                    .last()
                    .map(|entry| entry.id.clone())
                    .unwrap_or_default()
            } else {
                String::new()
            },
            entries: selected,
            problem: Default::default(),
            ..Default::default()
        })
    }

    fn flush(&self) {
        let (done, wait) = mpsc::sync_channel(1);

        if self.send(Job::Flush(done)) && wait.recv_timeout(FLUSH_TIMEOUT).is_err() {
            log::warn!("team audit: the writer did not catch up in time");
        }
    }
}

#[cfg(test)]
mod tests {
    use yunheshiguang_sdk::contract::wkt::Timestamp;
    use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::{
        AuditOutcome, GovernedAction,
    };
    use yunheshiguang_sdk::governance::Operator;

    use super::*;

    fn entry(id: &str, seconds: i64, operator: &str) -> AuditEntry {
        AuditEntry {
            id: id.to_string(),
            occurred_at: Timestamp {
                seconds,
                nanos: 0,
                ..Default::default()
            }
            .into(),
            action: GovernedAction::PairDevice.into(),
            outcome: AuditOutcome::Succeeded.into(),
            actor: "main".to_string(),
            operator: Operator {
                id: operator.to_string(),
                ..Operator::default()
            }
            .into(),
            ..AuditEntry::default()
        }
    }

    fn ids(page: &AuditPage) -> Vec<&str> {
        page.entries.iter().map(|entry| entry.id.as_str()).collect()
    }

    #[test]
    fn merges_every_computer_of_the_team_newest_first() {
        let folder = tempfile::tempdir().unwrap();
        let mine = TeamAuditStore::open(folder.path().to_path_buf(), "computer-a");
        let theirs = TeamAuditStore::open(folder.path().to_path_buf(), "computer-b");

        mine.append(&entry("a1", 100, "alice"));
        theirs.append(&entry("b1", 200, "bob"));
        mine.append(&entry("a2", 300, "alice"));
        theirs.flush();

        let all = mine.query(&AuditQuery::default()).unwrap();
        assert_eq!(ids(&all), ["a2", "b1", "a1"]);
        assert!(folder.path().join("computer-a.jsonl").is_file());
        assert!(folder.path().join("computer-b.jsonl").is_file());

        let bob = mine
            .query(&AuditQuery {
                operator_id: "bob".to_string(),
                ..AuditQuery::default()
            })
            .unwrap();
        assert_eq!(ids(&bob), ["b1"]);
    }

    #[test]
    fn survives_a_torn_last_line_and_bytes_that_are_not_utf8() {
        let folder = tempfile::tempdir().unwrap();
        let line = serde_json::to_string(&entry("old", 100, "alice")).unwrap();
        let mut torn = format!("{line}\n{{\"id\":\"half").into_bytes();
        torn.extend_from_slice(&[0xe4, 0xb8]);
        fs::write(folder.path().join("computer-a.jsonl"), torn).unwrap();
        let store = TeamAuditStore::open(folder.path().to_path_buf(), "computer-a");

        store.append(&entry("new", 200, "alice"));

        assert_eq!(
            ids(&store.query(&AuditQuery::default()).unwrap()),
            ["new", "old"]
        );
    }

    #[test]
    fn keeps_entries_it_could_not_write_and_writes_them_once_it_can() {
        let parent = tempfile::tempdir().unwrap();
        let folder = parent.path().join("synced");
        fs::write(&folder, "not a folder yet").unwrap();
        let store = TeamAuditStore::open(folder.clone(), "computer-a");

        store.append(&entry("early", 100, "alice"));
        store.flush();
        fs::remove_file(&folder).unwrap();
        fs::create_dir(&folder).unwrap();
        store.append(&entry("later", 200, "alice"));

        assert_eq!(
            ids(&store.query(&AuditQuery::default()).unwrap()),
            ["later", "early"]
        );
    }

    #[test]
    fn rotates_a_full_file_without_losing_what_it_held() {
        let folder = tempfile::tempdir().unwrap();
        let line = serde_json::to_string(&entry("old", 100, "alice")).unwrap();
        let mut full = format!("{line}\n").into_bytes();
        full.resize(usize::try_from(MAX_FILE_BYTES).unwrap() + 1, b'\n');
        fs::write(folder.path().join("computer-a.jsonl"), full).unwrap();
        let store = TeamAuditStore::open(folder.path().to_path_buf(), "computer-a");

        store.append(&entry("new", 200, "alice"));

        assert_eq!(
            ids(&store.query(&AuditQuery::default()).unwrap()),
            ["new", "old"]
        );
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 2);
    }

    #[test]
    fn pages_backwards_and_reads_nothing_from_an_empty_folder() {
        let folder = tempfile::tempdir().unwrap();
        let store = TeamAuditStore::open(folder.path().join("audit"), "me");
        assert!(
            store
                .query(&AuditQuery::default())
                .unwrap()
                .entries
                .is_empty()
        );

        for (seconds, id) in (1..).zip(["1", "2", "3"]) {
            store.append(&entry(id, seconds, "alice"));
        }
        let first = store
            .query(&AuditQuery {
                limit: 2,
                ..AuditQuery::default()
            })
            .unwrap();
        assert_eq!(
            (ids(&first), first.next_before_id.as_str()),
            (vec!["3", "2"], "2")
        );
        let rest = store
            .query(&AuditQuery {
                limit: 2,
                before_id: first.next_before_id,
                ..AuditQuery::default()
            })
            .unwrap();
        assert_eq!((ids(&rest), rest.next_before_id.as_str()), (vec!["1"], ""));
    }
}
