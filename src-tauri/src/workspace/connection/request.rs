use yunheshiguang_sdk::connection::{
    ConnectionProvider, Endpoint, EndpointError, NetworkSerial, PairingCode,
};

use crate::workspace::contract::{
    ConnectDeviceRequest, ConnectionResult, DisconnectDeviceRequest, GovernedAction,
    PairDeviceRequest, UiText,
};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

pub fn endpoint_error_text(error: EndpointError) -> UiText {
    ui_text(match error {
        EndpointError::InvalidHost => CONNECTION_INVALID_HOST_KEY,
        EndpointError::InvalidPort => CONNECTION_INVALID_PORT_KEY,
        EndpointError::InvalidPairingCode => CONNECTION_INVALID_PAIRING_CODE_KEY,
        EndpointError::InvalidDevice => CONNECTION_INVALID_DEVICE_KEY,
        _ => CONNECTION_INVALID_HOST_KEY,
    })
}

pub enum ConnectionRequest {
    Connect(Endpoint),
    Pair {
        endpoint: Endpoint,
        code: PairingCode,
    },
    Disconnect(NetworkSerial),
}

impl ConnectionRequest {
    pub fn connect(request: &ConnectDeviceRequest) -> Result<Self, UiText> {
        Endpoint::from_contract(request.endpoint.as_option())
            .map(Self::Connect)
            .map_err(endpoint_error_text)
    }

    pub fn pair(request: &PairDeviceRequest) -> Result<Self, UiText> {
        Ok(Self::Pair {
            endpoint: Endpoint::from_contract(request.endpoint.as_option())
                .map_err(endpoint_error_text)?,
            code: PairingCode::parse(&request.pairing_code).map_err(endpoint_error_text)?,
        })
    }

    pub fn disconnect(request: &DisconnectDeviceRequest) -> Result<Self, UiText> {
        NetworkSerial::parse(&request.device_id)
            .map(Self::Disconnect)
            .map_err(endpoint_error_text)
    }

    pub fn action(&self) -> GovernedAction {
        match self {
            Self::Connect(_) => GovernedAction::ConnectDevice,
            Self::Pair { .. } => GovernedAction::PairDevice,
            Self::Disconnect(_) => GovernedAction::DisconnectDevice,
        }
    }

    pub fn target(&self) -> String {
        match self {
            Self::Connect(endpoint) | Self::Pair { endpoint, .. } => endpoint.to_string(),
            Self::Disconnect(device) => device.as_str().to_string(),
        }
    }

    pub fn run(&self, provider: &dyn ConnectionProvider) -> ConnectionResult {
        match self {
            Self::Connect(endpoint) => provider.connect(endpoint),
            Self::Pair { endpoint, code } => provider.pair(endpoint, code),
            Self::Disconnect(device) => provider.disconnect(device),
        }
    }
}
