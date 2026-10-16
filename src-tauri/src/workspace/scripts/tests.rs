use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::time::Duration;

use yunheshiguang_sdk::device::{DeviceShell, ShellOutput};
use yunheshiguang_sdk::documents::{Collection, Document, DocumentId, DocumentStore};
use yunheshiguang_sdk::scripts::{
    ScriptEngine, ScriptEngineInfo, ScriptLanguage, ScriptOutcome, ScriptRequest,
};
use yunheshiguang_sdk::vendor::DeviceIdentity;

use super::engine::{MAX_OUTPUT_CHARS, SHELL_ENGINE_ID, ShellEngine, parse_output};
use super::library::{SCRIPTS_COLLECTION, load_script};
use super::runner::ScriptRunner;
use crate::workspace::contract::{Script, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

#[derive(Default)]
struct ScriptedShell {
    output: String,
    calls: RefCell<Vec<Vec<String>>>,
    pushed: RefCell<Vec<(String, String)>>,
    timeouts: RefCell<Vec<Duration>>,
}

impl DeviceShell for ScriptedShell {
    fn identity(&self) -> Option<&DeviceIdentity> {
        None
    }

    fn shell(&self, args: &[&str]) -> Result<ShellOutput, UiText> {
        self.calls
            .borrow_mut()
            .push(args.iter().map(ToString::to_string).collect());
        Ok(ShellOutput::new(""))
    }

    fn shell_with_timeout(&self, args: &[&str], timeout: Duration) -> Result<ShellOutput, UiText> {
        self.timeouts.borrow_mut().push(timeout);
        self.shell(args)?;
        Ok(ShellOutput::new(self.output.clone()))
    }

    fn push(&self, local: &Path, remote: &str) -> Result<(), UiText> {
        let body = fs::read_to_string(local).expect("pushed file should be readable");
        self.pushed.borrow_mut().push((remote.to_string(), body));
        Ok(())
    }
}

fn request<'a>(id: &'a str, language: ScriptLanguage, body: &'a str) -> ScriptRequest<'a> {
    ScriptRequest::new(id, "Test", language, body)
}

#[test]
fn runs_shell_scripts_from_a_pushed_file_and_reads_the_exit_code() {
    let device = ScriptedShell {
        output: "hello\nworld\n__yunheshiguang_exit__=3\n".to_string(),
        ..ScriptedShell::default()
    };

    let outcome = ShellEngine
        .run(
            &device,
            &request(
                "../evil id",
                ScriptLanguage::Shell,
                "echo hi\r\necho 'x'\r\n",
            ),
        )
        .expect("the script should run");

    assert_eq!(outcome.exit_code, Some(3));
    assert_eq!(outcome.output, "hello\nworld");
    assert!(!outcome.truncated);

    let pushed = device.pushed.borrow();
    let (remote, body) = &pushed[0];
    assert!(remote.starts_with("/data/local/tmp/yunheshiguang-scripts/evilid-"));
    assert!(remote.ends_with(".sh"));
    assert_eq!(body, "echo hi\necho 'x'\n");
    let stem = remote.trim_end_matches(".sh");

    let calls = device.calls.borrow();
    assert_eq!(
        calls[0],
        ["mkdir", "-p", "/data/local/tmp/yunheshiguang-scripts"]
    );
    assert_eq!(calls[1][..2], ["sh", "-c"]);
    assert!(calls[1][2].starts_with(&format!(
        "if command -v timeout >/dev/null 2>&1; then timeout -s KILL 290 sh {stem}.sh; else sh {stem}.sh; fi > {stem}.out 2>&1; code=$?;"
    )));
    assert!(calls[1][2].contains(&format!("rm -f {stem}.sh {stem}.out;")));
    assert!(device.timeouts.borrow()[0] >= Duration::from_secs(60));
}

#[test]
fn keeps_the_end_of_long_output_and_refuses_output_without_an_exit_code() {
    let long = format!(
        "{}\n__yunheshiguang_exit__=0",
        "x".repeat(MAX_OUTPUT_CHARS + 10)
    );
    let outcome = parse_output(&long).expect("the marker is present");

    assert_eq!(outcome.exit_code, Some(0));
    assert!(outcome.truncated);
    assert_eq!(outcome.output.chars().count(), MAX_OUTPUT_CHARS);
    assert_eq!(
        parse_output("__yunheshiguang_exit__=0"),
        Some(ScriptOutcome::finished(0, ""))
    );
    assert_eq!(parse_output("killed"), None);
    assert_eq!(
        parse_output("no newline at the end\n__yunheshiguang_exit__=1\n"),
        Some(ScriptOutcome::finished(1, "no newline at the end"))
    );

    let device = ScriptedShell {
        output: "Segmentation fault".to_string(),
        ..ScriptedShell::default()
    };
    assert_eq!(
        ShellEngine.run(&device, &request("s", ScriptLanguage::Shell, "true")),
        Err(ui_text(SCRIPT_NO_EXIT_CODE_KEY))
    );
}

struct Launcher(&'static str);

impl ScriptEngine for Launcher {
    fn describe(&self) -> ScriptEngineInfo {
        ScriptEngineInfo {
            id: self.0.to_string(),
            languages: vec![ScriptLanguage::Javascript.into()],
            ..ScriptEngineInfo::default()
        }
    }

    fn run(
        &self,
        _device: &dyn DeviceShell,
        _script: &ScriptRequest<'_>,
    ) -> Result<ScriptOutcome, UiText> {
        Ok(ScriptOutcome::launched())
    }
}

#[test]
fn hands_each_script_to_the_first_engine_that_takes_its_language() {
    let runner = ScriptRunner::new(
        vec![Box::new(Launcher("ext.js")), Box::new(Launcher("ext.js"))],
        vec![Box::new(ShellEngine)],
    );
    let ids = runner
        .engines()
        .engines
        .into_iter()
        .map(|engine| engine.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, ["ext.js", SHELL_ENGINE_ID]);

    let device = ScriptedShell {
        output: "__yunheshiguang_exit__=0".to_string(),
        ..ScriptedShell::default()
    };
    let javascript = runner.run(
        &device,
        &request("a", ScriptLanguage::Javascript, "toast(1)"),
    );
    assert_eq!(
        javascript.engine.map(|engine| engine.id).as_deref(),
        Some("ext.js")
    );
    assert_eq!(javascript.result, Ok(ScriptOutcome::launched()));

    let shell = runner.run(&device, &request("b", ScriptLanguage::Shell, "true"));
    assert_eq!(
        shell.engine.map(|engine| engine.id).as_deref(),
        Some(SHELL_ENGINE_ID)
    );

    let unknown = runner.run(&device, &request("c", ScriptLanguage::Unspecified, "?"));
    assert_eq!(unknown.engine, None);
    assert_eq!(unknown.result, Err(ui_text(SCRIPT_NO_ENGINE_KEY)));
}

fn save(store: &dyn DocumentStore, id: &str, body: &str) {
    store
        .save(
            &Collection::parse(SCRIPTS_COLLECTION).unwrap(),
            &Document::new(DocumentId::parse(id).unwrap(), body).unwrap(),
        )
        .unwrap();
}

#[test]
fn loads_scripts_from_the_library_documents() {
    let directory = tempfile::tempdir().unwrap();
    let store = yunheshiguang_sdk::documents::FolderDocumentStore::new(directory.path());
    save(
        &store,
        "hello",
        r#"{"name":"Hello","language":"SCRIPT_LANGUAGE_SHELL","body":"echo hi"}"#,
    );
    save(
        &store,
        "blank",
        r#"{"name":"Blank","language":"SCRIPT_LANGUAGE_SHELL","body":"  "}"#,
    );
    save(&store, "odd", r#"{"name":5}"#);

    assert_eq!(
        load_script(&store, "hello"),
        Ok(Script {
            name: "Hello".to_string(),
            language: ScriptLanguage::Shell.into(),
            body: "echo hi".to_string(),
            ..Default::default()
        })
    );
    assert_eq!(load_script(&store, "blank"), Err(ui_text(SCRIPT_EMPTY_KEY)));
    assert_eq!(
        load_script(&store, "odd"),
        Err(ui_text(SCRIPT_UNREADABLE_KEY))
    );
    assert_eq!(
        load_script(&store, "missing"),
        Err(ui_text(SCRIPT_MISSING_KEY))
    );
}
