use std::sync::Arc;

use yunheshiguang_sdk::connection::{ConnectionProvider, Endpoint, NetworkSerial, PairingCode};

use crate::workspace::adb::{ACTION_POLICY, AdbClient, AdbCommandError, AdbService};
use crate::workspace::android_tools::{AndroidPlatformToolsRegistry, probe_android_tools};
use crate::workspace::contract::ConnectionResult;

use super::adb_output::{
    command_failure, connect_result, disconnect_failure, disconnect_result, pair_result,
    tools_unavailable,
};

pub struct AdbConnectionProvider {
    registry: Arc<AndroidPlatformToolsRegistry>,
    adb: Arc<AdbService>,
}

impl AdbConnectionProvider {
    pub fn new(registry: Arc<AndroidPlatformToolsRegistry>, adb: Arc<AdbService>) -> Self {
        Self { registry, adb }
    }

    fn run(&self, args: &[&str]) -> Option<Result<String, AdbCommandError>> {
        self.run_with_input(args, None)
    }

    fn run_with_input(
        &self,
        args: &[&str],
        input: Option<&str>,
    ) -> Option<Result<String, AdbCommandError>> {
        let _runtime = self.adb.acquire_runtime();
        let adb_path = probe_android_tools(&self.registry.config()).adb_path?;
        let client = AdbClient::new(adb_path);
        let output = match input {
            Some(input) => client.run_with_input(args, input, ACTION_POLICY),
            None => client.run(args, ACTION_POLICY),
        };

        Some(output.map(|output| String::from_utf8_lossy(&output.stdout).into_owned()))
    }
}

impl ConnectionProvider for AdbConnectionProvider {
    fn connect(&self, endpoint: &Endpoint) -> ConnectionResult {
        match self.run(&["connect", &endpoint.to_string()]) {
            None => tools_unavailable(),
            Some(Ok(stdout)) => connect_result(&stdout),
            Some(Err(error)) => command_failure(&error),
        }
    }

    fn pair(&self, endpoint: &Endpoint, code: &PairingCode) -> ConnectionResult {
        let target = endpoint.to_string();

        match self.run_with_input(&["pair", &target], Some(&format!("{}\n", code.as_str()))) {
            None => tools_unavailable(),
            Some(Ok(stdout)) => pair_result(&stdout, &target),
            Some(Err(error)) => command_failure(&error),
        }
    }

    fn disconnect(&self, device: &NetworkSerial) -> ConnectionResult {
        match self.run(&["disconnect", device.as_str()]) {
            None => tools_unavailable(),
            Some(Ok(stdout)) => disconnect_result(&stdout, device.as_str()),
            Some(Err(error)) => disconnect_failure(&error, device.as_str()),
        }
    }
}
