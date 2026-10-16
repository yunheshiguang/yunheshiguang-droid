use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use yunheshiguang_contract::wkt::Timestamp;
use yunheshiguang_contract::yunheshiguang::updates::v1::{
    AvailableUpdate, BuildInfo as BuildMessage, UpdateAvailable, UpdateSettingsResult, UpdateState,
    UpdateStatus,
};
use yunheshiguang_contract::yunheshiguang::workspace::v1::UiText;

use crate::workspace::clock;
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::local_state::{read_json, write_json};
use crate::workspace::loopback::is_loopback_url;
use crate::workspace::messages::{
    UPDATES_STORAGE_FAILED_KEY, UPDATES_UNREACHABLE_KEY, UPDATES_UNREADABLE_KEY,
};
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::UPDATE_AVAILABLE_EVENT;
use super::build_info::{BuildInfo, Channel};
use super::install::InstallTarget;
use super::manifest::{self, Manifest};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_MANIFEST_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Settings {
    check_automatically: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            check_automatically: true,
        }
    }
}

#[derive(Default)]
struct Progress {
    status: Option<UpdateStatus>,
    update: Option<AvailableUpdate>,
    checked_at: Option<Timestamp>,
    problem: Option<UiText>,
    announced: Option<String>,
}

enum Outcome {
    UpToDate,
    Available(AvailableUpdate),
    Failed(UiText),
}

pub struct Updates {
    build: BuildInfo,
    target: InstallTarget,
    manifest_url: Option<String>,
    settings_path: PathBuf,
    settings: Mutex<Settings>,
    progress: Mutex<Progress>,
    checking: tokio::sync::Mutex<()>,
    events: Arc<dyn WorkspaceEvents>,
}

impl Updates {
    pub fn new(
        build: BuildInfo,
        target: InstallTarget,
        settings_path: PathBuf,
        events: Arc<dyn WorkspaceEvents>,
    ) -> Self {
        let manifest_url = build.manifest_url();
        Self::with_manifest_url(build, target, manifest_url, settings_path, events)
    }

    pub fn with_manifest_url(
        build: BuildInfo,
        target: InstallTarget,
        manifest_url: Option<String>,
        settings_path: PathBuf,
        events: Arc<dyn WorkspaceEvents>,
    ) -> Self {
        let settings = read_json(&settings_path).unwrap_or_default();

        Self {
            build,
            target,
            manifest_url,
            settings_path,
            settings: Mutex::new(settings),
            progress: Mutex::new(Progress::default()),
            checking: tokio::sync::Mutex::new(()),
            events,
        }
    }

    pub fn follows_a_channel(&self) -> bool {
        self.build.channel() != Channel::Local && self.manifest_url.is_some()
    }

    pub fn checks_automatically(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .check_automatically
    }

    pub fn state(&self) -> UpdateState {
        let progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
        let status = if self.follows_a_channel() {
            progress.status.unwrap_or(UpdateStatus::Idle)
        } else {
            UpdateStatus::LocalBuild
        };

        UpdateState {
            build: BuildMessage {
                channel: self.build.channel.clone(),
                version: self.build.version.clone(),
                commit: self.build.commit.clone(),
                repository: self.build.repository.clone(),
                ..Default::default()
            }
            .into(),
            check_automatically: self.checks_automatically(),
            status: status.into(),
            update: progress.update.clone().into(),
            checked_at: progress.checked_at.clone().into(),
            problem: progress.problem.clone().into(),
            releases_url: self.build.releases_url(),
            ..Default::default()
        }
    }

    pub fn set_automatic(&self, check_automatically: bool) -> UpdateSettingsResult {
        let settings = Settings {
            check_automatically,
        };
        let detail = match write_json(&self.settings_path, &settings) {
            Ok(()) => {
                *self.settings.lock().unwrap_or_else(PoisonError::into_inner) = settings;
                None
            }
            Err(error) => Some(ui_text_with_params(
                UPDATES_STORAGE_FAILED_KEY,
                "",
                [("message", error.to_string())],
            )),
        };

        UpdateSettingsResult {
            accepted: detail.is_none(),
            detail: detail.into(),
            state: self.state().into(),
            ..Default::default()
        }
    }

    pub async fn check(&self, announce: bool) -> UpdateState {
        let Some(url) = self
            .manifest_url
            .as_deref()
            .filter(|_| self.follows_a_channel())
        else {
            return self.state();
        };
        let _checking = self.checking.lock().await;
        self.progress
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .status = Some(UpdateStatus::Checking);

        let outcome = match fetch(url).await {
            Ok(None) => Outcome::UpToDate,
            Ok(Some(body)) => match manifest::parse(&body, &self.build.repository) {
                Some(manifest) => self
                    .newer(&manifest)
                    .map_or(Outcome::UpToDate, Outcome::Available),
                None => Outcome::Failed(ui_text(UPDATES_UNREADABLE_KEY)),
            },
            Err(message) => Outcome::Failed(ui_text_with_params(
                UPDATES_UNREACHABLE_KEY,
                "",
                [("message", message)],
            )),
        };
        let announcement = self.record(outcome, announce);
        if let Some(update) = announcement {
            notify(
                self.events.as_ref(),
                UPDATE_AVAILABLE_EVENT,
                &UpdateAvailable {
                    update: update.into(),
                    ..Default::default()
                },
            );
        }

        self.state()
    }

    fn record(&self, outcome: Outcome, announce: bool) -> Option<AvailableUpdate> {
        let mut progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
        progress.checked_at = Some(clock::now());
        match outcome {
            Outcome::UpToDate => {
                progress.status = Some(UpdateStatus::UpToDate);
                progress.update = None;
                progress.problem = None;
                None
            }
            Outcome::Available(update) => {
                progress.status = Some(UpdateStatus::Available);
                progress.problem = None;
                progress.update = Some(update.clone());
                let fresh = progress.announced.as_deref() != Some(update.id.as_str());
                if announce && fresh {
                    progress.announced = Some(update.id.clone());
                    Some(update)
                } else {
                    None
                }
            }
            Outcome::Failed(problem) => {
                progress.status = Some(UpdateStatus::Failed);
                progress.problem = Some(problem);
                None
            }
        }
    }

    fn newer(&self, manifest: &Manifest) -> Option<AvailableUpdate> {
        let newer = match self.build.channel() {
            Channel::Nightly => {
                manifest.channel == "nightly" && manifest.commit != self.build.commit
            }
            Channel::Release => {
                manifest.channel == "release" && is_newer(&manifest.version, &self.build.version)
            }
            Channel::Local => false,
        };
        if !newer {
            return None;
        }
        let asset = self.target.choose(manifest);
        let id = if manifest.channel == "nightly" {
            &manifest.commit
        } else {
            &manifest.tag
        };

        Some(AvailableUpdate {
            id: id.clone(),
            channel: manifest.channel.clone(),
            version: manifest.version.clone(),
            commit: manifest.commit.clone(),
            page_url: format!("{}/tag/{}", self.build.releases_url(), manifest.tag),
            download_url: asset.map(|asset| asset.url.clone()).unwrap_or_default(),
            download_name: asset.map(|asset| asset.name.clone()).unwrap_or_default(),
            ..Default::default()
        })
    }
}

pub fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |value: &str| semver::Version::parse(value.strip_prefix('v').unwrap_or(value)).ok();
    matches!(
        (parse(candidate), parse(current)),
        (Some(candidate), Some(current)) if candidate > current
    )
}

async fn fetch(url: &str) -> Result<Option<Vec<u8>>, String> {
    let mut client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(concat!("yunheshiguang/", env!("CARGO_PKG_VERSION")));
    if is_loopback_url(url) {
        client = client.no_proxy();
    } else {
        client = client.https_only(true);
    }
    let client = client.build().map_err(|error| error.to_string())?;
    let mut response = client
        .get(url)
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }

    let too_large = || format!("more than {} KB", MAX_MANIFEST_BYTES / 1024);
    if response
        .content_length()
        .is_some_and(|length| length > MAX_MANIFEST_BYTES as u64)
    {
        return Err(too_large());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if body.len() + chunk.len() > MAX_MANIFEST_BYTES {
            return Err(too_large());
        }
        body.extend_from_slice(&chunk);
    }

    Ok(Some(body))
}
