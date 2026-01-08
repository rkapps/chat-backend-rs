use std::env;
use anyhow::{Context, Result};

use agentic_core_rs::{
    agent::{Agent, builder::AgentBuilder},
    llm::{
        anthropic::{self, client::AnthropicClient}, client::LlmClient, gemini::{self, client::GeminiClient},
        openai::{self, client::OpenAIClient},
    },
};

use crate::agents::model::LlmProvider;

#[derive(Clone, Copy)]
pub struct AgentService {

}

impl AgentService {

    pub fn new() -> AgentService {
        Self {}
    }

    pub fn get_chat_agent(&self, llm: String, model: String) -> Result<Agent> {
        //get client
        let client = self.get_llm(&llm, model)?;
        // Build the Agent
        let agent = AgentBuilder::new()
            .client(client)
            .temperature(0.5)
            .max_tokens(5000)
            .build()?;

        Ok(agent)
    }

    pub fn get_llm(&self, llm: &str, model: String) -> Result<Box<dyn LlmClient>> {
        match llm {
            "Anthropic" => {
                let api_key = env::var("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY environment variable not set")?;
                let anthropic_version = env::var("ANTHROPIC_VERSION")
                    .context("ANTHROPIC_VERSION environment variable not set")?;

                Ok(Box::new(AnthropicClient::new(
                    api_key,
                    anthropic_version,
                    model,
                )))
            }
            "Gemini" => {
                let api_key = env::var("GEMINI_API_KEY")
                    .context("GEMINI_API_KEY environment variable not set")?;
                Ok(Box::new(GeminiClient::new(api_key, model)))
            }
            "OpenAI" => {
                let api_key = env::var("OPENAI_API_KEY")
                    .context("OPENAI_API_KEY environment variable not set")?;
                Ok(Box::new(OpenAIClient::new(api_key, model)))
            }

            _ => Err(anyhow::anyhow!("Llm client for '{}' is not supported", llm)),
        }
    }


    pub fn get_llm_providers(&self) -> Vec<LlmProvider>{
        let mut providers: Vec<LlmProvider> = Vec::new();

        let gemini = LlmProvider {
            id: String::from(gemini::client::LLM.to_lowercase()),
            llm: gemini::client::LLM.to_string(),
            models: vec![gemini::client::MODEL_GEMINI_3_FLASH_PREVIEW.to_string()],
        };
        let openai = LlmProvider{
            id: String::from(openai::client::LLM.to_lowercase()),
            llm: openai::client::LLM.to_string(),
            models: vec![openai::client::MODEL_GPT_5_NANO.to_string()],
        };
        let anthropic = LlmProvider{
            id: String::from(anthropic::client::LLM.to_lowercase()),
            llm: anthropic::client::LLM.to_string(),
            models: vec![anthropic::client::MODEL_CLAUDE_SONNET_4_5.to_string()],
        };
    
        providers.push(gemini);
        providers.push(openai);
        providers.push(anthropic);

        providers
    }

}
