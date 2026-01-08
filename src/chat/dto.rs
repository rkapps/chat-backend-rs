use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ChatErrorResponse {
    pub error: String,
    pub detail: String,
}


#[derive(Deserialize, Clone)]
pub struct ChatRequest {
    pub id: String,
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub role: String,
    pub content: Option<String>,
    pub response_id: Option<String>
}


