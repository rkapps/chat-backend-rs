use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::TokenUsage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentChunkResponse {
    /// Incremental visible text chunk — forwarded to UI
    Content { agent_id: String, content: String },
    /// Incremental thought/reasoning — not forwarded to UI
    Thought { agent_id: String, thought: String },
    // /// Pipeline status update — UI only
    // Status {
    //     agent_id: String,
    //     status: String,
    // },
    /// Agent execution complete — carries full agent response for storage
    Final { response: AgentResponse },
}

impl AgentChunkResponse {
    pub fn content(agent_id: String, content: String) -> Self {
        Self::Content { agent_id, content }
    }

    pub fn thought(agent_id: String, thought: String) -> Self {
        Self::Thought { agent_id, thought }
    }

    // pub fn status(agent_id: String, status: String) -> Self {
    //     Self::Status { agent_id, status }
    // }

    pub fn final_response(response: AgentResponse) -> Self {
        Self::Final { response }
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Final { .. })
    }

    pub fn agent_response(&self) -> Option<&AgentResponse> {
        match self {
            Self::Final { response } => Some(response),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub agent_id: String,
    pub model: String,
    pub iterations: Vec<AgentIteration>, // ← AgentIteration
    pub response_id: String,
    pub content: String,
    pub usage: TokenUsage,
    pub duration_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIteration {
    pub iteration: u32,
    pub tool_calls: Vec<AgentToolCall>,
    pub usage: TokenUsage,
    pub duration_ms: u64,
    pub response_id: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

// impl fmt::Display for AgentResponse {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "AgentResponse {{ agent: {}, model: {}, duration: {}ms, usage: {}, iterations: {} }}",
//             self.agent_id,
//             self.model,
//             self.duration_ms,
//             self.usage,
//             self.iterations.len()
//         )
//     }
// }
