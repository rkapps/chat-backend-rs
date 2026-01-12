use std::sync::Arc;

use agentic_core_rs::agent::service::{AgentService, LlmProvider};
use anyhow::Result;
use axum::{Json, extract::State};

use crate::chat::dto::ChatErrorResponse;


pub async fn llm_providers_handler(
    State(agent_service): State<Arc<AgentService>>,
) -> Result<Json<Vec<LlmProvider>>, Json<ChatErrorResponse>> {
    Ok(Json(agent_service.get_llm_providers()))
}
