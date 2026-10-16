use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::input::v1::{
    InputChannels, InputService, ListInputChannelsRequest,
};

use super::InputRouter;

pub struct InputRpc(pub Arc<InputRouter>);

impl InputService for InputRpc {
    async fn list_input_channels(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListInputChannelsRequest>,
    ) -> ServiceResult<InputChannels> {
        Response::ok(self.0.channels())
    }
}
