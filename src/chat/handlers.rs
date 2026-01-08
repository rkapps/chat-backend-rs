use std::{fmt::Display, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::debug;

use crate::{
    agents::service::AgentService,
    chat::{
        dto::{ChatErrorResponse, ChatRequest, ChatResponse},
        model::{Chat, ChatConfig},
        service::ChatService,
    },
};

pub async fn create_chat_handler(
    State(service): State<Arc<ChatService>>,
    Json(payload): Json<ChatConfig>,
) -> Result<Json<Chat>, (StatusCode, Json<ChatErrorResponse>)> {
    let chat = service
        .create_chat(payload)
        .await
        .map_err(|e| to_chat_error_response("Chat Error", e))?;
    Ok(Json(chat))
}

pub async fn get_all_chats_handler(
    State(service): State<Arc<ChatService>>,
) -> Result<Json<Vec<Chat>>, (StatusCode, Json<ChatErrorResponse>)> {
    let chats = service
        .get_all_chats()
        .await
        .map_err(|e| to_chat_error_response("Chat Error", e))?;
    Ok(Json(chats))
}

pub async fn get_chat_by_id_handler(
    State(service): State<Arc<ChatService>>,
    Path(id): Path<String>,
) -> Result<Json<Chat>, (StatusCode, Json<ChatErrorResponse>)> {
    let chat = service
        .get_chat_by_id(id)
        .await
        .map_err(|e| to_chat_error_response("Chat Error", e))?;
    Ok(Json(chat))
}

pub async fn chat_completion_handler(
    State(service): State<Arc<ChatService>>,
    State(agent_service): State<Arc<AgentService>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatErrorResponse>)> {
    //get chat

    let response = service
        .chat_completion(payload, *agent_service)
        .await
        .map_err(|e| to_chat_error_response("Chat completion error", e))?;
    Ok(Json(response))
}

fn to_chat_error_response(
    error: &str,
    detail: impl Display,
) -> (StatusCode, Json<ChatErrorResponse>) {
    debug!("Error: {:#?} Detail: {:#?}", error, detail.to_string());
    (
        StatusCode::BAD_REQUEST, // 400
        Json(ChatErrorResponse {
            error: error.to_string(),
            detail: detail.to_string(),
        }),
    )
}
