use rustic_agent::agents::domain::TurnChunkResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiChunk {
    pub id: String,
    pub content: String,
    pub thought: String,
    pub status: String,
    pub is_final: bool,
}

impl From<&TurnChunkResponse> for UiChunk {
    fn from(chunk: &TurnChunkResponse) -> Self {
        match chunk {
            TurnChunkResponse::Content { agent_id, content } => Self {
                id: agent_id.clone(),
                content: content.clone(),
                thought: String::new(),
                status: String::new(),
                is_final: false,
            },
            TurnChunkResponse::Thought { agent_id, thought } => Self {
                id: agent_id.clone(),
                content: String::new(),
                thought: thought.clone(),
                status: String::new(),
                is_final: false,
            },
            TurnChunkResponse::Status { agent_id, status } => Self {
                id: agent_id.clone(),
                content: String::new(),
                thought: String::new(),
                status: status.clone(),
                is_final: false,
            },
            TurnChunkResponse::Final { response } => Self {
                id: response.agent_id.clone(),
                content: String::new(),
                thought: String::new(),
                status: String::new(),
                is_final: true,
            },
        }
    }
}
