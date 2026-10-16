use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult, ServiceStream};
use futures_util::stream;
use tokio::sync::mpsc;
use yunheshiguang_contract::yunheshiguang::assistant::v1::{
    AssistantResult, AssistantService as AssistantApi, AssistantState, DraftFlowEvent,
    DraftFlowRequest, DraftScriptEvent, DraftScriptRequest, GetAssistantRequest,
    UpdateAssistantRequest, draft_flow_event, draft_script_event,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::principal;

use super::AssistantService;

fn events<T: Send + 'static>(receiver: mpsc::UnboundedReceiver<T>) -> ServiceStream<T> {
    Box::pin(stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (Ok(event), receiver))
    }))
}

pub struct AssistantRpc {
    pub service: Arc<AssistantService>,
    pub governance: Arc<Governance>,
}

impl AssistantApi for AssistantRpc {
    async fn get_assistant(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetAssistantRequest>,
    ) -> ServiceResult<AssistantState> {
        Response::ok(self.service.state())
    }

    async fn update_assistant(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, UpdateAssistantRequest>,
    ) -> ServiceResult<AssistantResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.update(request, &principal).await)
    }

    async fn draft_flow(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, DraftFlowRequest>,
    ) -> ServiceResult<ServiceStream<DraftFlowEvent>> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        let service = Arc::clone(&self.service);
        let (sender, receiver) = mpsc::unbounded_channel();

        tauri::async_runtime::spawn(async move {
            let written = sender.clone();
            let progress = move |piece: &str| {
                let _ = written.send(DraftFlowEvent {
                    event: Some(draft_flow_event::Event::Text(piece.to_string())),
                    ..Default::default()
                });
            };
            tokio::select! {
                result = service.draft_flow_with(request, &principal, Some(&progress)) => {
                    let _ = sender.send(DraftFlowEvent {
                        event: result.into(),
                        ..Default::default()
                    });
                }
                () = sender.closed() => {}
            }
        });

        Response::stream_ok(events(receiver))
    }

    async fn draft_script(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, DraftScriptRequest>,
    ) -> ServiceResult<ServiceStream<DraftScriptEvent>> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        let service = Arc::clone(&self.service);
        let (sender, receiver) = mpsc::unbounded_channel();

        tauri::async_runtime::spawn(async move {
            let written = sender.clone();
            let progress = move |piece: &str| {
                let _ = written.send(DraftScriptEvent {
                    event: Some(draft_script_event::Event::Text(piece.to_string())),
                    ..Default::default()
                });
            };
            tokio::select! {
                result = service.draft_script_with(request, &principal, Some(&progress)) => {
                    let _ = sender.send(DraftScriptEvent {
                        event: result.into(),
                        ..Default::default()
                    });
                }
                () = sender.closed() => {}
            }
        });

        Response::stream_ok(events(receiver))
    }
}
