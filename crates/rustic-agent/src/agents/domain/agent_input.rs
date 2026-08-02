use serde::{Deserialize, Serialize};

use crate::Preset;

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
