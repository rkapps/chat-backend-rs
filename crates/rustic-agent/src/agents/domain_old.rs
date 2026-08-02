use std::pin::Pin;

use futures::Stream;
use rustic_core::HttpResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Agent, Preset, TokenUsage};
/// All runtime parameters needed to build a [`Runnable`](super::runner::Runnable) for an agent.
///
/// Passed to [`AgentService::build_runnable`](crate::services::agent::AgentService::build_runnable)
/// and threaded down through recursive pipeline construction. LLM and model are inherited by
/// sub-agents; strategy and system prompt are overridden per-agent from their config.
///
///
/// A pinned, heap-allocated stream of [`CompletionChunkResponse`] items yielded by a streaming
/// completion call. Each item is wrapped in [`HttpResult`] to propagate transport-level errors
/// inline with the stream.
pub type TurnStreamResponse = Pin<Box<dyn Stream<Item = HttpResult<TurnChunkResponse>> + Send>>;

#[derive(Debug, Clone)]
pub struct AgentInput {
    pub agent_id: String,
    pub llm_config: LlmConfig,
    /// Optional system prompt override; `None` falls back to an empty string.
    pub system_prompt: Option<String>,
    /// Nested sub-agent inputs; empty for leaf agents, unused by `new()`.
    pub subs: Vec<AgentInput>,
    pub relay_tool_output: bool,
}

impl AgentInput {
    pub fn new(
        agent_id: String,
        llm_config: LlmConfig,
        system_prompt: Option<String>,
        relay_tool_output: bool,
    ) -> Self {
        Self {
            agent_id,
            llm_config,
            system_prompt,
            // strategy,
            subs: Vec::new(),
            relay_tool_output,
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct LlmConfig {
    /// Provider ID (e.g. `"anthropic"`, `"openai"`).
    pub llm: Option<String>,
    /// Model identifier forwarded to the provider (e.g. `"claude-sonnet-4-6"`).
    pub model: Option<String>,
    pub preset: Option<Preset>,
}

impl LlmConfig {
    /// Merge two configs — self takes priority, other fills in missing fields
    pub fn merge(self, other: LlmConfig) -> LlmConfig {
        LlmConfig {
            llm: self.llm.or(other.llm),
            model: self.model.or(other.model),
            preset: self.preset.or(other.preset),
        }
    }
}

/// A single completed exchange: the user prompt sent to an agent and the assistant reply received.
///
/// [`PipeLineAgent`](super::runner::PipeLineAgent) accumulates these across stages to build the
/// growing conversation history that is replayed to the orchestrator on each decision turn.
#[derive(Debug, Clone)]
pub struct CompletionTurn {
    /// Position of this turn in the pipeline (1-based).
    pub sequence: u32,
    pub user_content: String,
    pub response_content: String,
    /// Provider-assigned ID for the assistant response; used for multi-turn context continuations.
    pub response_id: Option<String>,
}

impl CompletionTurn {}

/// The orchestrator's parsed decision for a single pipeline stage.
///
/// The orchestrator LLM returns this as JSON. [`PipeLineAgent`](super::runner::PipeLineAgent)
/// deserialises it, runs the chosen sub-agents, and loops back to ask for the next decision —
/// unless `stop` is `true`, in which case the single nominated agent produces the final response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDecision {
    /// Sub-agents to run in this stage, each paired with a goal string.
    pub agents: Vec<AgentGoal>,
    pub execution: ExecutionMode,
    /// When `true` this is the synthesis stage; exactly one agent must be listed and
    /// execution must be sequential.
    pub stop: bool,
    /// Optional chain-of-thought from the orchestrator (useful for debugging).
    pub reasoning: Option<String>,
}

/// An agent nominated by the orchestrator for a stage, with an optional goal override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGoal {
    /// Must match the ID of a sub-agent registered in the pipeline config.
    pub id: String,
    /// Goal string forwarded as the prompt to the sub-agent; required when resolving.
    pub goal: Option<String>,
}

/// Controls whether agents in a stage run one-after-another or concurrently.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Agents execute in order; each receives the previous agent's output as context.
    Sequential,
    /// Agents execute concurrently (bounded by a semaphore of 5); results are merged afterwards.
    Parallel,
}

impl<'de> Deserialize<'de> for ExecutionMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "sequential" => Ok(ExecutionMode::Sequential),
            "parallel" => Ok(ExecutionMode::Parallel),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["sequential", "parallel"],
            )),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnLlmConfig {
    pub llm: String,
    pub max_tokens: i32,
    pub model: String,
    pub store: bool,
    pub temperature: f32,
}

impl TurnLlmConfig {
    pub fn new(
        llm: &str,
        model: &str,
        max_tokens: i32,
        store: bool,
        temperature: f32,
    ) -> TurnLlmConfig {
        Self {
            llm: llm.to_string(),
            model: model.to_string(),
            max_tokens,
            store,
            temperature,
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TurnStage {
//     pub name: String,
//     pub parallel: bool,
//     pub decision: StageDecision,
//     pub turn_responses: Vec<TurnResponse>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TurnChunkResponse {
    /// Incremental visible text chunk — forwarded to UI
    Content { agent_id: String, content: String },
    /// Incremental thought/reasoning — not forwarded to UI
    Thought { agent_id: String, thought: String },
    /// Pipeline status update — UI only
    Status { agent_id: String, status: String },
    /// Agent execution complete — carries full agent response for storage
    Final { response: TurnResponse },
}

impl TurnChunkResponse {
    pub fn content(agent_id: String, content: String) -> Self {
        Self::Content { agent_id, content }
    }

    pub fn thought(agent_id: String, thought: String) -> Self {
        Self::Thought { agent_id, thought }
    }

    pub fn status(agent_id: String, status: String) -> Self {
        Self::Status { agent_id, status }
    }

    pub fn final_response(response: TurnResponse) -> Self {
        Self::Final { response }
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Final { .. })
    }

    pub fn turn_response(&self) -> Option<&TurnResponse> {
        match self {
            Self::Final { response } => Some(response),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct TurnResponse {
    pub agent_id: String,
    pub prompt: String,
    pub content: String,             // final response content
    pub response_id: Option<String>, // LLM response id
    pub usage: TokenUsage,
    pub execution: TurnExecution, // pub stages: Vec<TurnStage>,
}

impl TurnResponse {
    pub fn for_single_agent(prompt: String, agent_response: AgentResponse) -> Self {
        let response = agent_response.clone();
        Self {
            agent_id: agent_response.agent_id,
            prompt,
            content: agent_response.content,
            response_id: Some(agent_response.response_id),
            usage: agent_response.usage,
            execution: TurnExecution::SingleAgent { response },
        }
    }
    pub fn for_deterministic(
        prompt: String,
        orchestrator_response: AgentResponse, // the LLM goal-setting call
    ) -> Self {
        Self {
            agent_id: orchestrator_response.agent_id.clone(),
            prompt,
            content: String::new(),
            response_id: None,
            usage: orchestrator_response.usage.clone(),
            execution: TurnExecution::Deterministic {
                orchestrator: orchestrator_response, // what the LLM decided (goals)
                stages: Vec::new(),
                synthesizer: None, // what actually ran (from config)
            },
        }
    }

    pub fn for_dynamic(agent_id: String, prompt: String) -> Self {
        Self {
            agent_id,
            prompt,
            content: String::new(),
            response_id: None,
            usage: TokenUsage::default(),
            execution: TurnExecution::Dynamic {
                decisions: Vec::new(),
                synthesizer: None,
            },
        }
    }

    pub fn add_stage(&mut self, stage: StageResponse) {
        if let TurnExecution::Deterministic { stages, .. } = &mut self.execution {
            // accumulate usage from all turn responses in the stage
            for turn in &stage.responses {
                self.usage += turn.usage.clone();
            }
            stages.push(stage);
        }
    }

    pub fn add_decision(&mut self, decision: DecisionResponse) {
        if let TurnExecution::Dynamic { decisions, .. } = &mut self.execution {
            // accumulate usage from all turn responses in the stage
            for turn in &decision.responses {
                self.usage += turn.usage.clone();
            }
            decisions.push(decision);
        }
    }

    pub fn set_synthesizer(&mut self, final_response: TurnResponse) {
        self.content = final_response.content.clone();
        self.usage += final_response.usage.clone();

        match &mut self.execution {
            TurnExecution::Deterministic { synthesizer, .. } => {
                *synthesizer = Some(Box::new(final_response));
            }
            TurnExecution::Dynamic { synthesizer, .. } => {
                *synthesizer = Some(Box::new(final_response));
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum TurnExecution {
    SingleAgent {
        response: AgentResponse,
    },
    Deterministic {
        orchestrator: AgentResponse, // LLM goal-setting — always iteration 0
        stages: Vec<StageResponse>,
        synthesizer: Option<Box<TurnResponse>>, // ← Box for recursion
    },
    Dynamic {
        decisions: Vec<DecisionResponse>,
        synthesizer: Option<Box<TurnResponse>>, // ← Box for recursion
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct StageResponse {
    pub name: String,
    pub responses: Vec<TurnResponse>,
    pub duration_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct DecisionResponse {
    pub iteration: u32,
    pub decision: StageDecision,
    pub responses: Vec<TurnResponse>,
    pub duration_ms: u64,
}
