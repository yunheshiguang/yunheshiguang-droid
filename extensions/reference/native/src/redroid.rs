use std::collections::HashMap;
use std::path::Path;

use bollard::errors::Error as DockerError;
use bollard::models::{ContainerSummary, ContainerSummaryStateEnum};
use bollard::query_parameters::{ListContainersOptionsBuilder, StopContainerOptionsBuilder};
use bollard::{API_DEFAULT_VERSION, Docker};
use serde::Deserialize;
use yunheshiguang_sdk::cloud::{
    CloudFuture, CloudPhone, CloudPhoneProvider, CloudPhoneStatus, CloudProviderInfo,
};
use yunheshiguang_sdk::contract::yunheshiguang::connection::v1::NetworkEndpoint;
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::UiText;
use yunheshiguang_sdk::text::{ui_text, ui_text_with_params};

use crate::config::read_json;
use crate::messages::*;
use crate::status::FileState;

pub const REDROID_PROVIDER_ID: &str = "reference.redroid";
const DEFAULT_LABEL: &str = "yunheshiguang.redroid";
const ADB_PORT: u16 = 5555;
const DOCKER_TIMEOUT_SECONDS: u64 = 30;
const LOCAL_SOCKET_SCHEMES: [&str; 2] = ["unix://", "npipe://"];
const HTTP_SCHEMES: [&str; 2] = ["tcp://", "http://"];
const STOP_GRACE_SECONDS: i32 = 10;

fn default_label() -> String {
    DEFAULT_LABEL.to_string()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RedroidFile {
    docker: String,
    adb_host: String,
    #[serde(default = "default_label")]
    label: String,
}

pub struct RedroidProvider {
    config: Result<RedroidFile, String>,
}

fn reachable(config: RedroidFile) -> Result<RedroidFile, String> {
    let address = config.docker.trim();
    let supported = !address.contains("://")
        || LOCAL_SOCKET_SCHEMES
            .iter()
            .chain(&HTTP_SCHEMES)
            .any(|scheme| address.starts_with(scheme));

    if supported {
        Ok(config)
    } else {
        Err(format!(
            "docker: {address} cannot be reached; use unix://, npipe://, tcp://, or http://"
        ))
    }
}

fn failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        REFERENCE_REDROID_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn status_of(state: Option<&ContainerSummaryStateEnum>) -> CloudPhoneStatus {
    match state {
        Some(ContainerSummaryStateEnum::RUNNING) => CloudPhoneStatus::Running,
        Some(ContainerSummaryStateEnum::CREATED | ContainerSummaryStateEnum::EXITED) => {
            CloudPhoneStatus::Stopped
        }
        Some(ContainerSummaryStateEnum::RESTARTING) => CloudPhoneStatus::Starting,
        Some(ContainerSummaryStateEnum::STOPPING | ContainerSummaryStateEnum::REMOVING) => {
            CloudPhoneStatus::Stopping
        }
        Some(ContainerSummaryStateEnum::DEAD) => CloudPhoneStatus::Failed,
        _ => CloudPhoneStatus::Unspecified,
    }
}

fn phone(summary: ContainerSummary, adb_host: &str) -> Option<CloudPhone> {
    let id = summary.id?;
    let port = summary
        .ports
        .unwrap_or_default()
        .into_iter()
        .find(|port| port.private_port == ADB_PORT)
        .and_then(|port| port.public_port);
    let name = summary
        .names
        .and_then(|names| names.into_iter().next())
        .map(|name| name.trim_start_matches('/').to_string())
        .unwrap_or_else(|| id.chars().take(12).collect());

    Some(CloudPhone {
        provider_id: String::new(),
        name,
        status: status_of(summary.state.as_ref()).into(),
        endpoint: port
            .map(|port| NetworkEndpoint {
                host: adb_host.to_string(),
                port: u32::from(port),
                ..Default::default()
            })
            .into(),
        detail: Default::default(),
        id,
        ..Default::default()
    })
}

impl RedroidProvider {
    pub fn load(path: &Path) -> (Option<Self>, FileState) {
        match read_json::<RedroidFile>(path).and_then(|config| config.map(reachable).transpose()) {
            Ok(None) => (None, FileState::Missing),
            Ok(Some(config)) => (Some(Self { config: Ok(config) }), FileState::Loaded),
            Err(reason) => (
                Some(Self {
                    config: Err(reason.clone()),
                }),
                FileState::Broken(reason),
            ),
        }
    }

    fn config(&self) -> Result<&RedroidFile, UiText> {
        self.config.as_ref().map_err(|reason| {
            ui_text_with_params(
                REFERENCE_REDROID_BROKEN_KEY,
                "",
                [("detail", reason.clone())],
            )
        })
    }

    fn docker(config: &RedroidFile) -> Result<Docker, UiText> {
        let address = config.docker.trim();
        let local_socket = LOCAL_SOCKET_SCHEMES
            .iter()
            .any(|scheme| address.starts_with(scheme));
        let connected = if local_socket {
            Docker::connect_with_socket(address, DOCKER_TIMEOUT_SECONDS, API_DEFAULT_VERSION)
        } else {
            Docker::connect_with_http(address, DOCKER_TIMEOUT_SECONDS, API_DEFAULT_VERSION)
        };

        connected.map_err(failed)
    }

    async fn list(&self) -> Result<Vec<CloudPhone>, UiText> {
        let config = self.config()?;
        let docker = Self::docker(config)?;
        let filters = HashMap::from([("label", vec![config.label.as_str()])]);
        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();
        let summaries = docker
            .list_containers(Some(options))
            .await
            .map_err(failed)?;

        Ok(summaries
            .into_iter()
            .filter_map(|summary| phone(summary, &config.adb_host))
            .collect())
    }

    async fn listed(&self, id: &str) -> Result<(), UiText> {
        if self.list().await?.iter().any(|phone| phone.id == id) {
            Ok(())
        } else {
            Err(ui_text(REFERENCE_REDROID_GONE_KEY))
        }
    }

    async fn after(
        &self,
        id: &str,
        changed: Result<(), DockerError>,
    ) -> Result<CloudPhone, UiText> {
        changed.map_err(failed)?;

        self.list()
            .await?
            .into_iter()
            .find(|phone| phone.id == id)
            .ok_or_else(|| ui_text(REFERENCE_REDROID_GONE_KEY))
    }
}

impl CloudPhoneProvider for RedroidProvider {
    fn describe(&self) -> CloudProviderInfo {
        CloudProviderInfo {
            id: REDROID_PROVIDER_ID.to_string(),
            label: ui_text(REFERENCE_REDROID_LABEL_KEY).into(),
            description: ui_text(REFERENCE_REDROID_DESCRIPTION_KEY).into(),
            can_start: true,
            can_stop: true,
            can_register: false,
            ..Default::default()
        }
    }

    fn phones(&self) -> CloudFuture<'_, Vec<CloudPhone>> {
        Box::pin(self.list())
    }

    fn start<'a>(&'a self, id: &'a str) -> CloudFuture<'a, CloudPhone> {
        Box::pin(async move {
            self.listed(id).await?;
            let docker = Self::docker(self.config()?)?;
            let started = docker.start_container(id, None).await;

            self.after(id, started).await
        })
    }

    fn stop<'a>(&'a self, id: &'a str) -> CloudFuture<'a, CloudPhone> {
        Box::pin(async move {
            self.listed(id).await?;
            let docker = Self::docker(self.config()?)?;
            let options = StopContainerOptionsBuilder::default()
                .t(STOP_GRACE_SECONDS)
                .build();
            let stopped = docker.stop_container(id, Some(options)).await;

            self.after(id, stopped).await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use wiremock::matchers::{method, path_regex, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn container(id: &str, state: &str, public_port: Option<u16>) -> serde_json::Value {
        serde_json::json!({
            "Id": id,
            "Names": [format!("/redroid-{id}")],
            "Image": "redroid/redroid:14.0.0-latest",
            "State": state,
            "Labels": { DEFAULT_LABEL: "" },
            "Ports": public_port.map_or_else(Vec::new, |port| vec![serde_json::json!({
                "IP": "0.0.0.0", "PrivatePort": 5555, "PublicPort": port, "Type": "tcp"
            })])
        })
    }

    fn provider(server: &MockServer) -> RedroidProvider {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("redroid.json");
        fs::write(
            &path,
            serde_json::json!({ "docker": server.uri(), "adbHost": "10.0.0.5" }).to_string(),
        )
        .unwrap();

        RedroidProvider::load(&path).0.unwrap()
    }

    #[test]
    fn lists_labelled_containers_as_cloud_phones() {
        tauri::async_runtime::block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path_regex(r"^(/v[\d.]+)?/containers/json$"))
                .and(query_param("all", "true"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    container("a1", "running", Some(40001)),
                    container("b2", "exited", None),
                ])))
                .mount(&server)
                .await;

            let phones = provider(&server).phones().await.unwrap();

            assert_eq!(phones.len(), 2);
            assert_eq!(phones[0].name, "redroid-a1");
            assert_eq!(phones[0].status, CloudPhoneStatus::Running);
            assert_eq!(
                phones[0].endpoint,
                NetworkEndpoint {
                    host: "10.0.0.5".to_string(),
                    port: 40001,
                    ..Default::default()
                }
                .into()
            );
            assert_eq!(phones[1].status, CloudPhoneStatus::Stopped);
            assert_eq!(phones[1].endpoint, Default::default());
            let sent = &server.received_requests().await.unwrap()[0];
            assert!(sent.url.query().unwrap().contains("yunheshiguang.redroid"));
        });
    }

    #[test]
    fn starts_a_container_and_reports_it_running() {
        tauri::async_runtime::block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path_regex(r"^(/v[\d.]+)?/containers/b2/start$"))
                .respond_with(ResponseTemplate::new(204))
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path_regex(r"^(/v[\d.]+)?/containers/json$"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    container("b2", "running", Some(40002)),
                ])))
                .mount(&server)
                .await;

            let started = provider(&server).start("b2").await.unwrap();

            assert_eq!(started.status, CloudPhoneStatus::Running);
            assert_eq!(started.endpoint.unwrap().port, 40002);
        });
    }

    #[test]
    fn leaves_containers_it_does_not_list_alone() {
        tauri::async_runtime::block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path_regex(r"^(/v[\d.]+)?/containers/.+/(start|stop)$"))
                .respond_with(ResponseTemplate::new(204))
                .expect(0)
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path_regex(r"^(/v[\d.]+)?/containers/json$"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    container("a1", "running", Some(40001)),
                ])))
                .mount(&server)
                .await;
            let provider = provider(&server);

            for refused in [provider.start("database").await, provider.stop("a").await] {
                assert_eq!(refused.unwrap_err().key, REFERENCE_REDROID_GONE_KEY);
            }
        });
    }

    #[test]
    fn explains_a_broken_file_or_an_unreachable_docker() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("redroid.json");
        fs::write(&path, r#"{ "docker": "tcp://x" }"#).unwrap();
        let (broken, state) = RedroidProvider::load(&path);
        assert!(matches!(state, FileState::Broken(_)));

        let listed = tauri::async_runtime::block_on(broken.unwrap().phones());
        assert_eq!(listed.unwrap_err().key, REFERENCE_REDROID_BROKEN_KEY);

        fs::write(
            &path,
            r#"{ "docker": "https://docker.example:2376", "adbHost": "h" }"#,
        )
        .unwrap();
        assert!(matches!(
            RedroidProvider::load(&path).1,
            FileState::Broken(reason) if reason.contains("cannot be reached")
        ));

        fs::write(
            &path,
            r#"{ "docker": "unix:///nonexistent/docker.sock", "adbHost": "h" }"#,
        )
        .unwrap();
        let unreachable = RedroidProvider::load(&path).0.unwrap();
        let listed = tauri::async_runtime::block_on(unreachable.phones());
        assert_eq!(listed.unwrap_err().key, REFERENCE_REDROID_FAILED_KEY);
    }
}
