pub mod chat;
pub mod helper;
pub mod llm;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct ChatConfig {
    pub llm: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
    pub system: Option<String>,
    pub prompt: Option<String>
}

#[derive(Deserialize, Clone)]
pub struct ChatRequest {
    pub id: String,
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    id: String,
    role: String,
    content: Option<String>,
    response_id: Option<String>
}

#[derive(Serialize)]
pub struct ChatErrorResponse {
    pub error: String,
    pub detail: String,
}


