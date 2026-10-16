use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use yunheshiguang_contract::yunheshiguang::updates::v1::{UpdateState, UpdateStatus};

use crate::workspace::events::WorkspaceEvents;
use crate::workspace::messages::{
    UPDATES_STORAGE_FAILED_KEY, UPDATES_UNREACHABLE_KEY, UPDATES_UNREADABLE_KEY,
};

use super::UPDATE_AVAILABLE_EVENT;
use super::build_info::{BuildInfo, Channel, DEFAULT_REPOSITORY};
use super::checker::{Updates, is_newer};
use super::install::{InstallTarget, LinuxInstall, linux_kind};
use super::manifest::{Manifest, parse};

const FIXTURE: &str = include_str!("../../../../scripts/release/fixtures/manifest.json");
const PUBLISHED: &str = "0123456789abcdef0123456789abcdef01234567";
const RUNNING: &str = "89abcdef0123456789abcdef0123456789abcdef";

#[derive(Default)]
struct RecordingEvents(Mutex<Vec<(String, Value)>>);

impl WorkspaceEvents for RecordingEvents {
    fn emit(&self, event: &'static str, payload: Value) {
        self.0.lock().unwrap().push((event.to_string(), payload));
    }
}

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn published(document: &Value) -> Manifest {
    parse(document.to_string().as_bytes(), DEFAULT_REPOSITORY).unwrap()
}

fn as_release(tag: &str, version: &str) -> Value {
    let mut document = fixture();
    document["channel"] = json!("release");
    document["tag"] = json!(tag);
    document["version"] = json!(version);
    for asset in document["assets"].as_array_mut().unwrap() {
        let url = asset["url"]
            .as_str()
            .unwrap()
            .replace("/nightly/", &format!("/{tag}/"));
        asset["url"] = json!(url);
    }
    document
}

struct Harness {
    server: MockServer,
    events: Arc<RecordingEvents>,
    updates: Updates,
    directory: tempfile::TempDir,
}

async fn harness(build: BuildInfo, target: InstallTarget) -> Harness {
    let server = MockServer::start().await;
    let directory = tempfile::tempdir().unwrap();
    let events = Arc::new(RecordingEvents::default());
    let updates = Updates::with_manifest_url(
        build,
        target,
        Some(format!("{}/release.json", server.uri())),
        directory.path().join("settings.json"),
        Arc::clone(&events) as Arc<dyn WorkspaceEvents>,
    );

    Harness {
        server,
        events,
        updates,
        directory,
    }
}

async fn serve(server: &MockServer, response: ResponseTemplate) {
    server.reset().await;
    Mock::given(method("GET"))
        .and(path("/release.json"))
        .respond_with(response)
        .mount(server)
        .await;
}

fn nightly() -> BuildInfo {
    BuildInfo::new("nightly", "0.1.0", RUNNING, "")
}

fn appimage() -> InstallTarget {
    InstallTarget::new("linux", "x86_64", Some("appimage"))
}

fn status(state: &UpdateState) -> Option<UpdateStatus> {
    state.status.as_known()
}

fn problem_key(state: &UpdateState) -> String {
    state
        .problem
        .as_option()
        .map(|text| text.key.clone())
        .unwrap_or_default()
}

#[test]
fn reads_the_manifest_the_release_scripts_publish() {
    let manifest = published(&fixture());

    assert_eq!(manifest.channel, "nightly");
    assert_eq!(manifest.tag, "nightly");
    assert_eq!(manifest.commit, PUBLISHED);
    assert_eq!(manifest.assets.len(), 6);
    assert!(
        manifest
            .assets
            .iter()
            .all(|asset| asset.url.ends_with(&asset.name))
    );
}

#[test]
fn rejects_manifests_it_cannot_trust() {
    let broken = [
        ("schema_version", json!(2)),
        ("channel", json!("beta")),
        ("commit", json!("0123456")),
        ("tag", json!("../nightly")),
        ("version", json!("")),
    ];
    for (field, value) in broken {
        let mut document = fixture();
        document[field] = value;
        assert!(
            parse(document.to_string().as_bytes(), DEFAULT_REPOSITORY).is_none(),
            "{field}"
        );
    }

    let mut missing = fixture();
    missing.as_object_mut().unwrap().remove("commit");
    assert!(parse(missing.to_string().as_bytes(), DEFAULT_REPOSITORY).is_none());
    assert!(parse(b"<html>", DEFAULT_REPOSITORY).is_none());
}

#[test]
fn drops_assets_that_do_not_belong_to_the_release() {
    let mut document = fixture();
    let assets = document["assets"].as_array_mut().unwrap();
    assets[0]["url"] = json!("https://example.com/yunheshiguang-windows-x86_64-setup.exe");
    assets[1]["name"] = json!("yunheshiguang-macos-arm64-renamed.dmg");
    assets[2]["sha256"] = json!("NOT-A-DIGEST");
    assets[3]["size"] = json!(0);
    assets[4]["kind"] = json!("Deb Package");
    assets.push(json!("not an asset"));

    let manifest = published(&document);
    let names: Vec<_> = manifest
        .assets
        .iter()
        .map(|asset| asset.name.as_str())
        .collect();
    assert_eq!(names, ["yunheshiguang-linux-x86_64.rpm"]);

    let elsewhere = parse(FIXTURE.as_bytes(), "someone/else").unwrap();
    assert!(elsewhere.assets.is_empty());
}

#[test]
fn orders_release_versions_by_semver() {
    assert!(is_newer("0.2.0", "0.1.0"));
    assert!(is_newer("v0.2.0", "0.1.0"));
    assert!(is_newer("0.2.0", "0.2.0-rc.1"));
    assert!(is_newer("0.10.0", "0.9.0"));
    assert!(!is_newer("0.1.0", "0.1.0"));
    assert!(!is_newer("0.2.0-rc.1", "0.2.0"));
    assert!(!is_newer("latest", "0.1.0"));
}

#[test]
fn only_builds_from_the_release_workflows_follow_a_channel() {
    let nightly = nightly();
    assert_eq!(nightly.channel(), Channel::Nightly);
    assert_eq!(
        nightly.manifest_url().as_deref(),
        Some(
            "https://github.com/yunheshiguang/yunheshiguang-android/releases/download/nightly/release.json"
        )
    );

    let release = BuildInfo::new("release", "0.1.0", PUBLISHED, "someone/fork");
    assert_eq!(release.channel(), Channel::Release);
    assert_eq!(
        release.manifest_url().as_deref(),
        Some("https://github.com/someone/fork/releases/latest/download/release.json")
    );

    for build in [
        BuildInfo::new("", "0.1.0", PUBLISHED, ""),
        BuildInfo::new("nightly", "0.1.0", "0123456", ""),
        BuildInfo::new("canary", "0.1.0", PUBLISHED, ""),
    ] {
        assert_eq!(build.channel(), Channel::Local);
        assert!(build.manifest_url().is_none());
    }
    assert_eq!(
        BuildInfo::new("release", "0.1.0", "", "../x").repository,
        DEFAULT_REPOSITORY
    );
}

#[test]
fn offers_the_file_that_matches_how_the_app_was_installed() {
    let manifest = published(&fixture());
    let chosen = |target: InstallTarget| target.choose(&manifest).map(|asset| asset.name.clone());

    let cases = [
        (
            ("windows", "x86_64", Some("installer")),
            Some("yunheshiguang-windows-x86_64-setup.exe"),
        ),
        (
            ("macos", "arm64", Some("disk-image")),
            Some("yunheshiguang-macos-arm64.dmg"),
        ),
        (
            ("macos", "x86_64", Some("disk-image")),
            Some("yunheshiguang-macos-x86_64.dmg"),
        ),
        (
            ("linux", "x86_64", Some("deb")),
            Some("yunheshiguang-linux-x86_64.deb"),
        ),
        (("linux", "x86_64", None), None),
        (("linux", "arm64", Some("appimage")), None),
    ];
    for ((os, arch, kind), expected) in cases {
        assert_eq!(
            chosen(InstallTarget::new(os, arch, kind)).as_deref(),
            expected,
            "{os} {arch}"
        );
    }

    let mut universal = manifest.clone();
    universal.assets[1].arch = "universal".to_string();
    universal.assets.remove(2);
    assert_eq!(
        InstallTarget::new("macos", "x86_64", Some("disk-image"))
            .choose(&universal)
            .map(|asset| asset.name.as_str()),
        Some("yunheshiguang-macos-arm64.dmg")
    );
}

#[test]
fn recognises_linux_packages() {
    let install = |appimage, executable: &str, dpkg, rpm| {
        linux_kind(LinuxInstall {
            appimage,
            executable: Some(Path::new(executable)),
            dpkg,
            rpm,
        })
    };

    assert_eq!(
        install(true, "/tmp/.mount_x/usr/bin/app", true, false),
        Some("appimage")
    );
    assert_eq!(
        install(false, "/usr/bin/yunheshiguang", true, true),
        Some("deb")
    );
    assert_eq!(
        install(false, "/usr/bin/yunheshiguang", false, true),
        Some("rpm")
    );
    assert_eq!(
        install(false, "/home/me/src/target/debug/app", true, false),
        None
    );
    assert_eq!(install(false, "/usr/bin/yunheshiguang", false, false), None);
}

#[tokio::test]
async fn announces_each_new_nightly_once() {
    let harness = harness(nightly(), appimage()).await;
    serve(
        &harness.server,
        ResponseTemplate::new(200).set_body_string(FIXTURE),
    )
    .await;

    let state = harness.updates.check(true).await;
    assert_eq!(status(&state), Some(UpdateStatus::Available));
    let update = state.update.as_option().unwrap();
    assert_eq!(update.id, PUBLISHED);
    assert_eq!(update.channel, "nightly");
    assert_eq!(update.download_name, "yunheshiguang-linux-x86_64.AppImage");
    assert_eq!(
        update.page_url,
        "https://github.com/yunheshiguang/yunheshiguang-android/releases/tag/nightly"
    );
    assert!(state.checked_at.is_set());

    harness.updates.check(true).await;
    harness.updates.check(false).await;
    let events = harness.events.0.lock().unwrap().clone();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, UPDATE_AVAILABLE_EVENT);
    assert_eq!(events[0].1["update"]["id"], PUBLISHED);
}

#[tokio::test]
async fn the_published_nightly_is_up_to_date_with_itself() {
    let harness = harness(
        BuildInfo::new("nightly", "0.1.0", PUBLISHED, ""),
        appimage(),
    )
    .await;
    serve(
        &harness.server,
        ResponseTemplate::new(200).set_body_string(FIXTURE),
    )
    .await;

    let state = harness.updates.check(true).await;
    assert_eq!(status(&state), Some(UpdateStatus::UpToDate));
    assert!(state.update.as_option().is_none());
    assert!(harness.events.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn release_builds_follow_newer_versions() {
    let outdated = harness(BuildInfo::new("release", "0.1.0", RUNNING, ""), appimage()).await;
    let body = as_release("v0.2.0", "0.2.0").to_string();
    serve(
        &outdated.server,
        ResponseTemplate::new(200).set_body_string(body),
    )
    .await;

    let state = outdated.updates.check(false).await;
    assert_eq!(status(&state), Some(UpdateStatus::Available));
    let update = state.update.as_option().unwrap();
    assert_eq!(
        (update.id.as_str(), update.version.as_str()),
        ("v0.2.0", "0.2.0")
    );
    assert!(
        update
            .download_url
            .ends_with("/releases/download/v0.2.0/yunheshiguang-linux-x86_64.AppImage")
    );

    let current = harness(BuildInfo::new("release", "0.2.0", RUNNING, ""), appimage()).await;
    let body = as_release("v0.2.0", "0.2.0").to_string();
    serve(
        &current.server,
        ResponseTemplate::new(200).set_body_string(body),
    )
    .await;
    assert_eq!(
        status(&current.updates.check(false).await),
        Some(UpdateStatus::UpToDate)
    );
}

#[tokio::test]
async fn a_channel_with_nothing_published_is_up_to_date() {
    let harness = harness(nightly(), appimage()).await;
    serve(&harness.server, ResponseTemplate::new(404)).await;

    let state = harness.updates.check(true).await;
    assert_eq!(
        status(&state),
        Some(UpdateStatus::UpToDate),
        "{:?}",
        state.problem
    );
    assert!(state.problem.as_option().is_none());
}

#[tokio::test]
async fn says_why_a_check_failed() {
    let harness = harness(nightly(), appimage()).await;

    serve(&harness.server, ResponseTemplate::new(503)).await;
    let state = harness.updates.check(true).await;
    assert_eq!(status(&state), Some(UpdateStatus::Failed));
    assert_eq!(problem_key(&state), UPDATES_UNREACHABLE_KEY);

    serve(
        &harness.server,
        ResponseTemplate::new(200).set_body_string("{}"),
    )
    .await;
    assert_eq!(
        problem_key(&harness.updates.check(true).await),
        UPDATES_UNREADABLE_KEY
    );

    let oversized = format!("{{\"padding\":\"{}\"}}", "x".repeat(300 * 1024));
    serve(
        &harness.server,
        ResponseTemplate::new(200).set_body_string(oversized),
    )
    .await;
    let state = harness.updates.check(true).await;
    assert_eq!(problem_key(&state), UPDATES_UNREACHABLE_KEY);
    let message = &state.problem.as_option().unwrap().params["message"];
    assert!(message.contains("256 KB"), "{message}");

    serve(
        &harness.server,
        ResponseTemplate::new(200).set_body_string(FIXTURE),
    )
    .await;
    let state = harness.updates.check(true).await;
    assert_eq!(status(&state), Some(UpdateStatus::Available));
    assert!(state.problem.as_option().is_none());
}

#[tokio::test]
async fn local_builds_do_not_check() {
    let directory = tempfile::tempdir().unwrap();
    let updates = Updates::new(
        BuildInfo::new("", "0.1.0", "", ""),
        appimage(),
        directory.path().join("settings.json"),
        Arc::new(RecordingEvents::default()),
    );

    assert!(!updates.follows_a_channel());
    assert_eq!(status(&updates.state()), Some(UpdateStatus::LocalBuild));
    assert_eq!(
        status(&updates.check(true).await),
        Some(UpdateStatus::LocalBuild)
    );
    assert_eq!(
        updates.state().releases_url,
        "https://github.com/yunheshiguang/yunheshiguang-android/releases"
    );
}

#[tokio::test]
async fn remembers_whether_to_check_automatically() {
    let harness = harness(nightly(), appimage()).await;
    let settings: PathBuf = harness.directory.path().join("settings.json");
    assert!(harness.updates.state().check_automatically);

    let result = harness.updates.set_automatic(false);
    assert!(result.accepted);
    assert!(!result.state.as_option().unwrap().check_automatically);

    let reopened = Updates::new(
        nightly(),
        appimage(),
        settings,
        Arc::new(RecordingEvents::default()),
    );
    assert!(!reopened.checks_automatically());
}

#[test]
fn reports_a_setting_it_could_not_save() {
    let directory = tempfile::tempdir().unwrap();
    let blocker = directory.path().join("blocker");
    std::fs::write(&blocker, "").unwrap();
    let updates = Updates::new(
        nightly(),
        appimage(),
        blocker.join("settings.json"),
        Arc::new(RecordingEvents::default()),
    );

    let result = updates.set_automatic(false);
    assert!(!result.accepted);
    assert_eq!(
        result.detail.as_option().unwrap().key,
        UPDATES_STORAGE_FAILED_KEY
    );
    assert!(result.state.as_option().unwrap().check_automatically);
    assert!(updates.checks_automatically());
}
