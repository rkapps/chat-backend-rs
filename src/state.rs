use std::sync::Arc;
use axum::extract::FromRef;

use crate::{chat::service::ChatService};

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatService>,
}

impl FromRef<AppState> for Arc<ChatService> {
    fn from_ref(state: &AppState) -> Self {
        state.chat_service.clone()
    }
}

