use std::sync::Arc;
use axum::extract::FromRef;

use crate::{agents::service::AgentService, chat::service::ChatService};

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatService>,
    pub agent_service: Arc<AgentService>,
}

impl FromRef<AppState> for Arc<ChatService> {
    fn from_ref(state: &AppState) -> Self {
        state.chat_service.clone()
    }
}

impl FromRef<AppState> for Arc<AgentService> {
    fn from_ref(state: &AppState) -> Self {
        state.agent_service.clone()
    }
}