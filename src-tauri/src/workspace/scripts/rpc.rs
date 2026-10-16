use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::scripts::v1::{
    ListScriptEnginesRequest, ScriptEngines, ScriptService,
};

use super::ScriptRunner;

pub struct ScriptsRpc(pub Arc<ScriptRunner>);

impl ScriptService for ScriptsRpc {
    async fn list_script_engines(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListScriptEnginesRequest>,
    ) -> ServiceResult<ScriptEngines> {
        Response::ok(self.0.engines())
    }
}
