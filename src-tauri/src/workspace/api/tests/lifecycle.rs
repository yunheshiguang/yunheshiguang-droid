use connectrpc::Protocol;
use connectrpc::client::{ClientConfig, HttpClient};
use yunheshiguang_contract::yunheshiguang::workspace::v1::WorkspaceServiceClient;

use super::*;

#[test]
fn relays_events_only_while_open() {
    let relay = EventRelay::default();
    relay.publish("workspace:automation-changed", "{}");
    assert!(relay.subscribe().is_none());

    relay.open();
    let mut receiver = relay.subscribe().unwrap();
    relay.publish("workspace:automation-changed", r#"{"runId":"r"}"#);
    let event = receiver.try_recv().unwrap();
    assert_eq!(event.event, "workspace:automation-changed");
    assert_eq!(event.payload_json, r#"{"runId":"r"}"#);

    relay.close();
    assert!(matches!(
        receiver.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Closed)
    ));
}

#[test]
fn keeps_only_digests_of_the_tokens_it_issues() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tokens.json");
    let store = TokenStore::load(path.clone());

    let (token, secret) = store.create("  Home Assistant ").unwrap();
    assert_eq!(token.name, "Home Assistant");
    assert!(secret.starts_with(TOKEN_PREFIX));
    assert!(secret.len() > 40);
    let file = std::fs::read_to_string(&path).unwrap();
    assert!(!file.contains(&secret[TOKEN_PREFIX.len()..]));

    let reloaded = TokenStore::load(path);
    assert_eq!(
        reloaded.authenticate(&secret).as_deref(),
        Some("Home Assistant")
    );
    assert!(reloaded.list()[0].last_used_at.is_set());
    assert_eq!(reloaded.authenticate("yhsg_guess"), None);
    assert_eq!(
        reloaded.revoke(&token.id).unwrap().as_deref(),
        Some("Home Assistant")
    );
    assert_eq!(reloaded.authenticate(&secret), None);
    assert_eq!(reloaded.revoke(&token.id).unwrap(), None);

    assert_eq!(
        reloaded.create(" ").unwrap_err(),
        ui_text(API_TOKEN_NAME_REQUIRED_KEY)
    );
    reloaded.create("ci").unwrap();
    assert_eq!(
        reloaded.create(" CI ").unwrap_err(),
        ui_text(API_TOKEN_NAME_TAKEN_KEY)
    );
    for index in 1..MAX_TOKENS {
        reloaded.create(&format!("t{index}")).unwrap();
    }
    assert_eq!(
        reloaded.create("one too many").unwrap_err().key,
        API_TOKEN_LIMIT_KEY
    );
}

#[test]
fn asks_extension_authenticators_before_the_core_tokens() {
    let tokens = Arc::new(TokenStore::in_memory());
    let authenticators = Authenticators::new(
        vec![Box::new(EnterpriseKeys), Box::new(EnterpriseKeys)],
        vec![Box::new(CoreTokens(tokens))],
    );
    let ids = authenticators
        .describe()
        .into_iter()
        .map(|info| info.id)
        .collect::<Vec<_>>();

    assert_eq!(ids, ["test.enterprise", CORE_TOKENS_ID]);
    assert_eq!(
        authenticators.identify("enterprise-key").unwrap().name,
        "ops"
    );
    assert!(authenticators.identify("").is_none());
}

fn free_port() -> u16 {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn http_get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn listens_on_the_loopback_interface_only_while_enabled() {
    let gateway = ApiGateway::new(api().context);
    let port = free_port();
    let config = |enabled| ApiGatewayConfig {
        enabled,
        port: u32::from(port),
        ..Default::default()
    };

    let running = tauri::async_runtime::block_on(gateway.apply(&config(true)));
    assert_eq!(running.status, ApiGatewayStatus::Running);
    assert_eq!(running.address, format!("http://127.0.0.1:{port}"));
    assert!(http_get(port, "/v1/health").starts_with("HTTP/1.1 200"));
    assert!(http_get(port, SNAPSHOT).starts_with("HTTP/1.1 401"));

    let blocked = ApiGateway::new(api().context);
    let failed = tauri::async_runtime::block_on(blocked.apply(&config(true)));
    assert_eq!(failed.status, ApiGatewayStatus::Failed);
    assert_eq!(failed.problem.unwrap().key, API_BIND_FAILED_KEY);

    let stopped = tauri::async_runtime::block_on(gateway.apply(&config(false)));
    assert_eq!(stopped.status, ApiGatewayStatus::Stopped);
    assert_eq!(gateway.status(), stopped);
}

#[test]
fn speaks_connect_grpc_and_grpc_web_on_one_port() {
    let api = api();
    let gateway = ApiGateway::new(api.context.clone());
    let port = free_port();
    let config = |enabled| ApiGatewayConfig {
        enabled,
        port: u32::from(port),
        ..Default::default()
    };

    tauri::async_runtime::block_on(async {
        assert_eq!(
            gateway.apply(&config(true)).await.status,
            ApiGatewayStatus::Running
        );
        let uri: axum::http::Uri = format!("http://127.0.0.1:{port}").parse().unwrap();

        for (protocol, transport) in [
            (Protocol::Connect, HttpClient::plaintext()),
            (Protocol::GrpcWeb, HttpClient::plaintext()),
            (Protocol::Grpc, HttpClient::plaintext_http2_only()),
        ] {
            let client = WorkspaceServiceClient::new(
                transport,
                ClientConfig::new(uri.clone())
                    .with_protocol(protocol)
                    .with_default_header("authorization", format!("Bearer {}", api.secret)),
            );
            let snapshot = client
                .get_workspace_snapshot(GetWorkspaceSnapshotRequest::default())
                .await
                .unwrap_or_else(|error| panic!("{protocol:?}: {error}"))
                .into_owned();

            assert_eq!(snapshot.schema_version, "test", "{protocol:?}");
        }

        gateway.apply(&config(false)).await;
    });
}
