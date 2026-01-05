use std::env;
use agentic_core_rs::{agent::{Agent, builder::AgentBuilder}, llm::{anthropic::client::AnthropicClient, client::LlmClient, gemini::client::GeminiClient, openai::client::OpenAIClient}};
use anyhow::{Context, Result};



pub fn get_agent(llm: String, model: String) -> Result<Agent> {
    //get client
    let client = get_llm(&llm, model)?;
    // Build the Agent
    let agent = AgentBuilder::new()
        .client(client)
        .temperature(0.5)
        .max_tokens(5000)
        .build()?
        ;

    Ok(agent)
}


pub fn get_llm(llm: &str, model: String) -> Result<Box<dyn LlmClient>> {
    match llm {
        "Anthropic" => {
            let api_key =
                env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY environment variable not set")?;
            let anthropic_version =
                env::var("ANTHROPIC_VERSION").context("ANTHROPIC_VERSION environment variable not set")?;
            
            Ok(Box::new(AnthropicClient::new(api_key, anthropic_version, model)))
        }
        "Gemini" => {
            // let model =
            //     env::var("GEMINI_MODEL").context("GEMINI_MODEL environment variable not set")?;
            let api_key =
                env::var("GEMINI_API_KEY").context("GEMINI_API_KEY environment variable not set")?;
            Ok(Box::new(GeminiClient::new(api_key, model)))
        }
        "OpenAI" => {
            // let model =
            //     env::var("OPENAI_MODEL").context("OPENAI_MODEL environment variable not set")?;
            let api_key =
                env::var("OPENAI_API_KEY").context("OPENAI_API_KEY environment variable not set")?;
            Ok(Box::new(OpenAIClient::new(api_key, model)))
        }

        _ => Err(anyhow::anyhow!("Llm client for '{}' is not supported", llm)),
    }

}


