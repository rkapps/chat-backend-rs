use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct ChatRequest {
    pub id: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatStreamingMessage {
    pub id: String,
    pub user_content: String,
    pub assistant_content: String,
    pub response_id: String
}


#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub role: String,
    pub content: Option<String>,
    pub response_id: Option<String>
}


#[derive(Serialize)]
pub struct ChatErrorResponse {
    pub error: String,
    pub detail: String,
}
