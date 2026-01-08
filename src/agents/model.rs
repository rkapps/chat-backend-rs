use serde::Serialize;


#[derive(Serialize)]
pub struct LlmProvider {
    pub id: String,
    pub llm: String,
    pub models: Vec<String>,
}
