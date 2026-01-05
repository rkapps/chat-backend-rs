use std::{fmt::Display};

use agentic_core_rs::capabilities::{completion::CompletionRequest, messages::MessageRole};
use anyhow::Result;
use axum::{Json, http::StatusCode, extract::Path};
use tracing::debug;

use crate::{
    handlers::{ChatConfig, ChatErrorResponse, ChatRequest, ChatResponse, helper::get_agent},
    storage::{
        Chat, add_chat_to_map, add_chatrequest_to_map, add_chatresponse_to_map, get_chat_by_id, get_chats,
    },
};

pub async fn get_chats_handler (
) -> Result<Json<Vec<Chat>>, (StatusCode, Json<ChatErrorResponse>)> {

    let chats = get_chats()
        .map_err(|e| to_chat_error_response("Chat Completion error", &e))?;

    Ok(Json(chats))
}


pub async fn get_chat_by_id_handler (Path(id): Path<String>)-> Result<Json<Chat>, (StatusCode, Json<ChatErrorResponse>)> {

    let chat = get_chat_by_id(id)
        .map_err(|e| to_chat_error_response("Chat Completion error", &e))?;

    Ok(Json(chat))
}


pub async fn chat_create_handler(
    Json(payload): Json<ChatConfig>,
) -> Result<Json<Chat>, (StatusCode, Json<ChatErrorResponse>)> {
    let chat =
        add_chat_to_map(payload).map_err(|e| to_chat_error_response("Chat create error", &e))?;
    debug!("Chat created with id: {:#?}", chat.clone().id);
    Ok(Json(chat))
}

pub async fn chat_completion_handler(
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ChatErrorResponse>)> {
    let chat = get_chat_by_id(payload.clone().id)
        .map_err(|e| to_chat_error_response("Chat Completion error", &e))?;
    let agent = get_agent(chat.llm, chat.model)
        .map_err(|e| to_chat_error_response("AgentBuilder error", &e))?;

    let schat = add_chatrequest_to_map(payload)
        .map_err(|e| to_chat_error_response("AgentBuilder error", &e))?;
    let chat_id = schat.id;

    //Create the completion request
    let request = CompletionRequest {
        model: agent.config.client.model().to_string(),
        system: schat.system,
        messages: schat.messages,
        max_tokens: agent.max_tokens(),
        temperature: agent.temperature(),
    };

    // On error, Send an error response
    let cresponse = agent
        .complete(request)
        .await
        .map_err(|e| to_chat_error_response("Completion Response error", &e))?;

    //Create the chat response message and add it the chat.
    let message = cresponse.content;
    let response_id = Some(cresponse.id);

    add_chatresponse_to_map(&chat_id, message.clone(), response_id.clone())
        .map_err(|e| to_chat_error_response("Chat Response error", &e))?;

    //Create the chat response
    let response = ChatResponse {
        id: chat_id,
        role: MessageRole::Assistant.as_str().to_string(),
        content: Some(message),
        response_id: response_id,
    };
    debug!("Chat Request: {:#?}", response);

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
