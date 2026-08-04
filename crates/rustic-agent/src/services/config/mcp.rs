use anyhow::{self, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::tools::mcp::MCPServerSetting;

/// Configuration for a single MCP server entry in `mcp_servers_config.json`.
///
/// API keys are not stored directly — `api_key_env` names the environment variable
/// that holds the actual key, which is resolved at startup by [`to_core_config`](Self::to_core_config).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    /// Logical server name used to namespace tool definitions (e.g. `"weather"`).
    pub name: String,
    /// HTTP endpoint URL of the MCP server.
    pub url: String,
    /// Name of the environment variable that holds the Bearer token for this server.
    pub api_key_env: String,
    /// Optional header name to use
    pub auth_header_name: Option<String>,
    /// Tool names to register from this server; only these tools are exposed to agents.
    pub enabled_tools: Vec<String>,
}

impl MCPServerConfig {
    /// Resolve the API key from the environment and return an [`MCPServerSetting`].
    ///
    /// Returns an error if `api_key_env` is not set in the environment.
    pub fn to_core_config(&self) -> Result<MCPServerSetting> {
        let api_key = std::env::var(&self.api_key_env)
            .map_err(|_| anyhow::anyhow!("Env var {} not set", self.api_key_env))?;
        debug!("api_key {:?} : {:?}", self.api_key_env, api_key);

        Ok(MCPServerSetting {
            name: self.name.clone(),
            url: self.url.clone(),
            api_key,
            auth_header_name: self.auth_header_name.clone(),
        })
    }
}
