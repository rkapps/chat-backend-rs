use agentic_core::agent::service::LlmProvider;
use anyhow::Result;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Sse, sse::Event},
};
use futures::StreamExt;
use std::{convert::Infallible, fmt::Display, sync::Arc};
use tracing::debug;

use crate::chat::{
    dto::{ChatErrorResponse, ChatRequest, ChatResponse, ChatStreamingMessage},
    model::{Chat, ChatConfig},
    service::ChatService,
};

pub async fn create_chat_handler(
    State(chat_service): State<Arc<ChatService>>,
    Json(payload): Json<ChatConfig>,
) -> Result<Json<Chat>, (StatusCode, Json<ChatErrorResponse>)> {
    debug!("config: {:?}", payload);
    let chat = chat_service
        .create_chat(payload)
        .await
        .map_err(|e| to_chat_error_response("Chat Error", e))?;
    Ok(Json(chat))
}

pub async fn get_all_chats_handler(
    State(chat_service): State<Arc<ChatService>>,
) -> Result<Json<Vec<Chat>>, (StatusCode, Json<ChatErrorResponse>)> {
    let chats = chat_service
        .get_all_chats()
        .await
        .map_err(|e| to_chat_error_response("Chat Error", e))?;
    Ok(Json(chats))
}

pub async fn get_chat_by_id_handler(
    State(chat_service): State<Arc<ChatService>>,
    Path(id): Path<String>,
) -> Result<Json<Chat>, (StatusCode, Json<ChatErrorResponse>)> {
    let chat = chat_service
        .get_chat_by_id(id)
        .await
        .map_err(|e| to_chat_error_response("Chat Error", e))?;

    debug!("get chat by id handler {:?}", chat);

    Ok(Json(chat))
}

pub async fn chat_completion_handler(
    State(chat_service): State<Arc<ChatService>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatErrorResponse>)> {
    //get chat
    debug!("started chat_completion_streaming_handler {:?}", payload);

    let response = chat_service
        .chat_completion(payload)
        .await
        .map_err(|e| to_chat_error_response("Chat completion error", e))?;
    Ok(Json(response))
}

pub async fn chat_completion_streaming_handler(
    State(chat_service): State<Arc<ChatService>>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    debug!("started chat_completion_streaming_handler");

    let stream = match chat_service
        .chat_completion_streaming(payload.clone())
        .await
    {
        Ok(stream) => stream,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let event_stream = stream.map(move |chunk_result| {
        match chunk_result {
            Ok(chunk) => {
                // Convert your ChatResponseChunk to SSE Event
                debug!("chunk: {:?}", chunk);

                match serde_json::to_string(&chunk) {
                    Ok(c) => {
                        Ok::<Event, Infallible>(
                            Event::default()
                                .data(c) // Serialize to JSON string
                                .event("message"),
                        )
                    }
                    Err(e) => Ok::<Event, Infallible>(
                        Event::default().data(format!("{}", e)).event("error"),
                    ),
                }
            }
            Err(e) => {
                // Send error as SSE event
                debug!("error: {:?}", e);
                Ok::<Event, Infallible>(Event::default().data(format!("{}", e)).event("error"))
            }
        }
    });

    Sse::new(event_stream).into_response()
}

pub async fn save_streaming_message_handler(
    State(chat_service): State<Arc<ChatService>>,
    Json(payload): Json<ChatStreamingMessage>,
) -> Result<(), (StatusCode, Json<ChatErrorResponse>)> {
    //get chat
    debug!("started chat_completion_streaming_handler {:?}", payload);

    chat_service
        .save_streaming_message(payload)
        .await
        .map_err(|e| to_chat_error_response("Chat save streaming message error", e))?;

    Ok(())
}

pub async fn llm_providers_handler(
    State(chat_service): State<Arc<ChatService>>,
) -> Result<Json<Vec<LlmProvider>>, Json<ChatErrorResponse>> {
    Ok(Json(chat_service.get_llm_providers()))
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
