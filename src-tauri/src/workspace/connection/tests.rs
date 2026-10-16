use std::sync::{Arc, Mutex};
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_contract::yunheshiguang::connection::v1::NetworkEndpoint;
use yunheshiguang_sdk::connection::{ConnectionProvider, Endpoint, NetworkSerial, PairingCode};
use yunheshiguang_sdk::governance::{ActionContext, AuthorizationProvider, Decision};

use super::adb_output::{
    command_failure, connect_result, disconnect_failure, disconnect_result, pair_result,
};
use super::outcome::connection_result;
use super::request::ConnectionRequest;
use super::rpc::govern;
use crate::workspace::adb::{AdbCommandError, AdbErrorKind};
use crate::workspace::contract::*;
use crate::workspace::governance::{Governance, GovernanceParts};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

fn outcome(result: &ConnectionResult) -> ConnectionOutcome {
    ConnectionOutcome::known(result.outcome).unwrap()
}

fn detail_key(result: &ConnectionResult) -> &str {
    &result.detail.as_option().unwrap().key
}

#[test]
fn explains_each_invalid_request_with_its_own_copy() {
    let endpoint = |host: &str, port| {
        Some(NetworkEndpoint {
            host: host.to_string(),
            port,
            ..Default::default()
        })
    };
    let key = |request: Result<ConnectionRequest, UiText>| request.err().unwrap().key;

    let connect = |host: &str, port| ConnectDeviceRequest {
        endpoint: endpoint(host, port).into(),
        scope: Default::default(),
        ..Default::default()
    };

    assert_eq!(
        key(ConnectionRequest::connect(&connect(
            "192.168.001.020",
            5555
        ))),
        CONNECTION_INVALID_HOST_KEY
    );
    assert_eq!(
        key(ConnectionRequest::connect(&connect("192.168.1.20", 0))),
        CONNECTION_INVALID_PORT_KEY
    );
    assert_eq!(
        key(ConnectionRequest::pair(&PairDeviceRequest {
            endpoint: endpoint("192.168.1.20", 37099).into(),
            pairing_code: "12".to_string(),
            scope: Default::default(),
            ..Default::default()
        })),
        CONNECTION_INVALID_PAIRING_CODE_KEY
    );
    assert_eq!(
        key(ConnectionRequest::disconnect(&DisconnectDeviceRequest {
            device_id: "R58M20ABCDE".to_string(),
            scope: Default::default(),
            ..Default::default()
        })),
        CONNECTION_INVALID_DEVICE_KEY
    );
}

#[test]
fn reads_the_connect_verdict_from_stdout() {
    let connected = connect_result(
        "* daemon not running; starting now at tcp:5037\n* daemon started successfully\nconnected to 192.168.1.20:5555\n",
    );
    assert_eq!(outcome(&connected), ConnectionOutcome::Connected);
    assert_eq!(connected.device_id, "192.168.1.20:5555");

    let again = connect_result("already connected to 192.168.1.20:5555");
    assert_eq!(outcome(&again), ConnectionOutcome::AlreadyConnected);
    assert_eq!(again.device_id, "192.168.1.20:5555");

    let refused = connect_result("failed to connect to '127.0.0.1:1': Connection refused\n");
    assert_eq!(outcome(&refused), ConnectionOutcome::Unreachable);
    assert_eq!(
        refused.detail.as_option().unwrap().params["reason"],
        "failed to connect to '127.0.0.1:1': Connection refused"
    );

    let unresolved = connect_result(
        "failed to resolve host: 'no-such-host.invalid': nodename nor servname provided, or not known",
    );
    assert_eq!(outcome(&unresolved), ConnectionOutcome::Unreachable);

    let unauthorized = connect_result("failed to authenticate to 192.168.1.20:5555");
    assert_eq!(outcome(&unauthorized), ConnectionOutcome::Rejected);
    assert_eq!(detail_key(&unauthorized), CONNECTION_UNAUTHORIZED_KEY);

    assert_eq!(outcome(&connect_result("")), ConnectionOutcome::Failed);
}

#[test]
fn reads_pairing_results_and_failures() {
    let paired = pair_result(
        "Successfully paired to 192.168.1.20:37099 [guid=adb-XYZ]\n",
        "192.168.1.20:37099",
    );
    assert_eq!(outcome(&paired), ConnectionOutcome::Paired);
    assert_eq!(
        paired.detail.as_option().unwrap().params["endpoint"],
        "192.168.1.20:37099"
    );
    let prompted = pair_result(
        "Enter pairing code: Successfully paired to 192.168.1.20:37099 [guid=adb-XYZ]\n",
        "192.168.1.20:37099",
    );
    assert_eq!(outcome(&prompted), ConnectionOutcome::Paired);

    let wrong_code = command_failure(&AdbCommandError::non_zero(
        Some(1),
        "Failed: Wrong password or connection was dropped.",
    ));
    assert_eq!(outcome(&wrong_code), ConnectionOutcome::Rejected);
    assert_eq!(detail_key(&wrong_code), CONNECTION_PAIRING_REJECTED_KEY);

    let not_listening = command_failure(&AdbCommandError::non_zero(
        Some(1),
        "error: protocol fault (couldn't read status message): Undefined error: 0",
    ));
    assert_eq!(outcome(&not_listening), ConnectionOutcome::Unreachable);

    let timeout = command_failure(&AdbCommandError::new(AdbErrorKind::Timeout, "timed out"));
    assert_eq!(outcome(&timeout), ConnectionOutcome::Unreachable);

    let missing = command_failure(&AdbCommandError::new(AdbErrorKind::NotFound, "no adb"));
    assert_eq!(outcome(&missing), ConnectionOutcome::ToolsUnavailable);
}

#[test]
fn reads_disconnect_results() {
    let disconnected = disconnect_result("disconnected 192.168.1.20:5555\n", "192.168.1.20:5555");
    assert_eq!(outcome(&disconnected), ConnectionOutcome::Disconnected);
    assert_eq!(disconnected.device_id, "192.168.1.20:5555");

    let unknown = disconnect_failure(
        &AdbCommandError::non_zero(Some(1), "error: no such device '127.0.0.1:1'"),
        "127.0.0.1:1",
    );
    assert_eq!(outcome(&unknown), ConnectionOutcome::NotConnected);
    assert_eq!(unknown.device_id, "127.0.0.1:1");
}

#[test]
fn keeps_the_pairing_code_out_of_the_audited_target() {
    let request = ConnectionRequest::pair(&PairDeviceRequest {
        endpoint: NetworkEndpoint {
            host: "192.168.1.20".to_string(),
            port: 37099,
            ..Default::default()
        }
        .into(),
        pairing_code: "123456".to_string(),
        scope: Default::default(),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(request.action(), GovernedAction::PairDevice);
    assert_eq!(request.target(), "192.168.1.20:37099");
}

struct FakeProvider {
    calls: Mutex<usize>,
    result: ConnectionResult,
}

impl FakeProvider {
    fn returning(outcome: ConnectionOutcome) -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(0),
            result: connection_result(outcome, "192.168.1.20:5555", ui_text("test.detail")),
        })
    }

    fn answer(&self) -> ConnectionResult {
        *self.calls.lock().unwrap() += 1;
        self.result.clone()
    }
}

impl ConnectionProvider for FakeProvider {
    fn connect(&self, _endpoint: &Endpoint) -> ConnectionResult {
        self.answer()
    }

    fn pair(&self, _endpoint: &Endpoint, _code: &PairingCode) -> ConnectionResult {
        self.answer()
    }

    fn disconnect(&self, _device: &NetworkSerial) -> ConnectionResult {
        self.answer()
    }
}

struct DenyAll;

impl AuthorizationProvider for DenyAll {
    fn describe(&self) -> GovernanceProvider {
        GovernanceProvider::default()
    }

    fn authorize(&self, _context: &ActionContext<'_>) -> Decision {
        Decision::Deny(ui_text("test.denied"))
    }
}

fn connect_request() -> ConnectionRequest {
    ConnectionRequest::Connect(Endpoint::parse("192.168.1.20", 5555).unwrap())
}

fn run(governance: &Arc<Governance>, provider: Arc<FakeProvider>) -> ConnectionResult {
    tauri::async_runtime::block_on(govern(
        governance,
        provider,
        &governance.principal("main"),
        connect_request(),
    ))
}

#[test]
fn audits_a_successful_connection() {
    let governance = Arc::new(Governance::core());
    let provider = FakeProvider::returning(ConnectionOutcome::Connected);

    let result = run(&governance, Arc::clone(&provider));

    assert_eq!(outcome(&result), ConnectionOutcome::Connected);
    assert_eq!(*provider.calls.lock().unwrap(), 1);

    let entry = &governance.state().audit_trail[0];
    assert_eq!(entry.action, GovernedAction::ConnectDevice);
    assert_eq!(entry.outcome, AuditOutcome::Succeeded);
    assert_eq!(entry.targets, vec!["***5555"]);
}

#[test]
fn audits_a_failed_connection_as_a_failure() {
    let governance = Arc::new(Governance::core());

    let result = run(
        &governance,
        FakeProvider::returning(ConnectionOutcome::Unreachable),
    );

    assert_eq!(outcome(&result), ConnectionOutcome::Unreachable);
    assert_eq!(
        governance.state().audit_trail[0].outcome,
        AuditOutcome::Failed
    );
}

#[test]
fn never_reaches_the_provider_when_governance_denies() {
    let governance = Arc::new(Governance::assemble(GovernanceParts {
        authorization: Some(Box::new(DenyAll)),
        ..GovernanceParts::default()
    }));
    let provider = FakeProvider::returning(ConnectionOutcome::Connected);

    let result = run(&governance, Arc::clone(&provider));

    assert_eq!(outcome(&result), ConnectionOutcome::Denied);
    assert_eq!(detail_key(&result), "test.denied");
    assert_eq!(*provider.calls.lock().unwrap(), 0);
    assert_eq!(
        governance.state().audit_trail[0].outcome,
        AuditOutcome::Denied
    );
}
