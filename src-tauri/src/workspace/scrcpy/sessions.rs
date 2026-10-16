use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::workspace::adb::configure_command;
use crate::workspace::contract::{ScrcpyDiagnosticCode, ScrcpySession, ScrcpySessionStatus};

use super::diagnostics::ScrcpyProblem;

const KEPT_LINES: usize = 4;
const READER_GRACE: Duration = Duration::from_millis(200);

type Lease = Arc<Mutex<Option<Box<dyn Send + Sync>>>>;

struct Running {
    session: ScrcpySession,
    child: Option<Child>,
    output: Arc<Mutex<VecDeque<String>>>,
    reader: Option<JoinHandle<()>>,
    lease: Lease,
}

pub struct Launch<'a> {
    pub client: &'a Path,
    pub server: Option<&'a Path>,
    pub adb: &'a Path,
    pub args: Vec<String>,
    pub serial: &'a str,
    pub window_label: &'a str,
    pub lease: Box<dyn Send + Sync>,
}

#[derive(Default)]
pub struct Sessions {
    running: Mutex<Vec<Running>>,
}

fn last_words(output: &Mutex<VecDeque<String>>, serial: &str) -> String {
    output
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|line| line.replace(serial, "<device>"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn release(lease: &Lease) {
    lease.lock().unwrap_or_else(PoisonError::into_inner).take();
}

fn keep_output(
    child: &mut Child,
    lease: &Lease,
) -> (Arc<Mutex<VecDeque<String>>>, Option<JoinHandle<()>>) {
    let output = Arc::new(Mutex::new(VecDeque::new()));
    let reader = child.stderr.take().and_then(|stderr| {
        let kept = Arc::clone(&output);
        let held = Arc::clone(lease);
        thread::Builder::new()
            .name("scrcpy-output".to_string())
            .spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut line = Vec::new();
                while reader
                    .read_until(b'\n', &mut line)
                    .is_ok_and(|read| read > 0)
                {
                    let text = String::from_utf8_lossy(&line)
                        .trim_end_matches(['\n', '\r'])
                        .to_string();
                    line.clear();
                    let mut kept = kept.lock().unwrap_or_else(PoisonError::into_inner);
                    if kept.len() == KEPT_LINES {
                        kept.pop_front();
                    }
                    kept.push_back(text);
                }
                release(&held);
            })
            .ok()
    });

    (output, reader)
}

impl Running {
    fn refresh(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        let Ok(Some(status)) = child.try_wait() else {
            return;
        };

        if let Some(reader) = self.reader.take() {
            let mut waited = Duration::ZERO;
            while !reader.is_finished() && waited < READER_GRACE {
                thread::sleep(Duration::from_millis(10));
                waited += Duration::from_millis(10);
            }
        }
        self.child = None;
        release(&self.lease);
        self.session.pid = 0;
        if status.success() {
            self.session.status = ScrcpySessionStatus::Idle.into();
        } else {
            self.session.status = ScrcpySessionStatus::Failed.into();
            self.session.last_error_code = ScrcpyDiagnosticCode::LaunchFailed.into();
            self.session.last_error = last_words(&self.output, &self.session.serial);
        }
    }

    fn is_running(&self) -> bool {
        self.child.is_some()
    }
}

impl Sessions {
    fn running(&self) -> std::sync::MutexGuard<'_, Vec<Running>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn list(&self) -> Vec<ScrcpySession> {
        let mut running = self.running();
        running.iter_mut().for_each(Running::refresh);
        running.retain(|entry| entry.session.status != ScrcpySessionStatus::Idle);
        running.iter().map(|entry| entry.session.clone()).collect()
    }

    pub fn launch(&self, launch: Launch<'_>) -> Result<ScrcpySession, ScrcpyProblem> {
        let mut running = self.running();
        running.iter_mut().for_each(Running::refresh);
        if running
            .iter()
            .any(|entry| entry.session.serial == launch.serial && entry.is_running())
        {
            return Err(ScrcpyProblem::new(ScrcpyDiagnosticCode::AlreadyRunning, ""));
        }
        running.retain(|entry| entry.session.serial != launch.serial);

        let mut command = Command::new(launch.client);
        command
            .args(&launch.args)
            .env("ADB", launch.adb)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if let Some(server) = launch.server {
            command.env("SCRCPY_SERVER_PATH", server);
        }
        configure_command(&mut command);

        let mut child = command.spawn().map_err(|error| {
            ScrcpyProblem::new(ScrcpyDiagnosticCode::LaunchFailed, error.to_string())
        })?;
        let session = ScrcpySession {
            id: uuid::Uuid::now_v7().to_string(),
            window_label: launch.window_label.to_string(),
            serial: launch.serial.to_string(),
            status: ScrcpySessionStatus::Running.into(),
            pid: child.id(),
            last_error: String::new(),
            last_error_code: ScrcpyDiagnosticCode::None.into(),
            ..Default::default()
        };
        let lease = Arc::new(Mutex::new(Some(launch.lease)));
        let (output, reader) = keep_output(&mut child, &lease);

        running.push(Running {
            session: session.clone(),
            child: Some(child),
            output,
            reader,
            lease,
        });
        Ok(session)
    }

    pub fn serial_of(&self, session_id: &str) -> Option<String> {
        self.running()
            .iter()
            .find(|entry| entry.session.id == session_id)
            .map(|entry| entry.session.serial.clone())
    }

    fn position(running: &[Running], session_id: &str, serial: &str) -> Option<usize> {
        running.iter().position(|entry| {
            if session_id.is_empty() {
                entry.session.serial == serial
            } else {
                entry.session.id == session_id
            }
        })
    }

    pub fn contains(&self, session_id: &str, serial: &str) -> bool {
        Self::position(&self.running(), session_id, serial).is_some()
    }

    pub fn stop(&self, session_id: &str, serial: &str) -> Result<(), ScrcpyProblem> {
        let mut running = self.running();
        let Some(index) = Self::position(&running, session_id, serial) else {
            return Ok(());
        };

        if let Some(child) = running[index].child.as_mut() {
            child.kill().map_err(|error| {
                ScrcpyProblem::new(ScrcpyDiagnosticCode::StopFailed, error.to_string())
            })?;
            let _ = child.wait();
        }
        running.remove(index);
        Ok(())
    }
}

impl Sessions {
    pub fn any_running(&self) -> bool {
        let mut running = self.running();
        running.iter_mut().for_each(Running::refresh);
        running.iter().any(Running::is_running)
    }

    pub fn stop_all(&self) {
        for mut entry in self.running().drain(..) {
            if let Some(child) = entry.child.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

impl Drop for Sessions {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn launch<'a>(script: &'a Path, serial: &'a str) -> Launch<'a> {
        Launch {
            client: script,
            server: None,
            adb: Path::new("/bin/false"),
            args: Vec::new(),
            serial,
            window_label: "main",
            lease: Box::new(()),
        }
    }

    fn script(body: &str) -> tempfile::TempPath {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), format!("#!/bin/sh\n{body}\n")).unwrap();
        crate::workspace::tool_archive::ensure_executable(file.path()).unwrap();
        file.into_temp_path()
    }

    fn wait_until_listed_as(
        sessions: &Sessions,
        status: ScrcpySessionStatus,
    ) -> Vec<ScrcpySession> {
        for _ in 0..200 {
            let listed = sessions.list();
            if listed.iter().all(|session| session.status == status) {
                return listed;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("sessions never became {status:?}");
    }

    #[test]
    fn keeps_one_window_per_device_and_closes_it_on_request() {
        let sleeper = script("sleep 30");
        let sessions = Sessions::default();

        let started = sessions.launch(launch(&sleeper, "abc")).unwrap();
        assert_eq!(started.status, ScrcpySessionStatus::Running);
        assert_eq!(
            sessions.launch(launch(&sleeper, "abc")).unwrap_err().code,
            ScrcpyDiagnosticCode::AlreadyRunning
        );

        sessions.stop(&started.id, "").unwrap();
        assert!(sessions.list().is_empty());
    }

    #[test]
    fn reports_what_scrcpy_said_when_it_fails() {
        let failing = script("echo 'ERROR: Could not find device abc' >&2; exit 1");
        let sessions = Sessions::default();

        sessions.launch(launch(&failing, "abc")).unwrap();
        let listed = wait_until_listed_as(&sessions, ScrcpySessionStatus::Failed);

        assert_eq!(
            listed[0].last_error,
            "ERROR: Could not find device <device>"
        );
        assert_eq!(listed[0].pid, 0);
    }

    #[test]
    fn lets_installs_proceed_once_a_window_has_failed() {
        let failing = script("exit 1");
        let sessions = Sessions::default();

        sessions.launch(launch(&failing, "abc")).unwrap();
        wait_until_listed_as(&sessions, ScrcpySessionStatus::Failed);

        assert!(!sessions.any_running());
    }

    #[test]
    fn lets_go_of_adb_as_soon_as_scrcpy_exits() {
        struct Flag(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Flag {
            fn drop(&mut self) {
                self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let quitting = script("exit 0");
        let released = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sessions = Sessions::default();

        sessions
            .launch(Launch {
                lease: Box::new(Flag(Arc::clone(&released))),
                ..launch(&quitting, "abc")
            })
            .unwrap();

        for _ in 0..200 {
            if released.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(released.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn forgets_a_window_the_operator_closed() {
        let quitting = script("exit 0");
        let sessions = Sessions::default();

        sessions.launch(launch(&quitting, "abc")).unwrap();

        assert!(wait_until_listed_as(&sessions, ScrcpySessionStatus::Idle).is_empty());
    }
}
