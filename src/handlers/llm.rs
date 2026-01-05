use agentic_core_rs::llm::{anthropic, gemini, openai};
use axum::Json;
use serde::Serialize;

use crate::handlers::ChatErrorResponse;

#[derive(Serialize)]
pub struct LlmProvider {
    id: String,
    llm: String,
    models: Vec<String>,
}

pub async fn llm_providers_handler() -> Result<Json<Vec<LlmProvider>>, Json<ChatErrorResponse>> {
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

    Ok(Json(providers))
}
