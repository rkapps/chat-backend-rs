use agentic_core_rs::capabilities::messages::Message;
use serde::{Deserialize, Serialize};
use storage_core::core::RepoModel;
use uuid::Uuid;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub llm: String,
    pub model: String,
    pub system: Option<String>,
    pub prompt: String,
    pub messages: Vec<Message>
}

impl RepoModel<String> for Chat {
    fn id(&self) -> String {
        self.clone().id
    }
    fn collection(&self) -> &'static str {
        "chat"
    }
   
}

impl Chat {

    pub fn new(llm: String, model: String, title: String, system: Option<String>, prompt: String) -> Self {
        let id = Uuid::new_v4().to_string();
        Self{id: id, title: title, llm: llm, model: model, system: system, prompt: prompt, messages: Vec::new()}
    }

}



#[derive(Deserialize)]
pub struct ChatConfig {
    pub llm: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
    pub system: Option<String>,
    pub prompt: Option<String>
}

impl ChatConfig {
    
    pub fn validate(self) -> Result<Chat>{

        let llm = self.llm.ok_or_else(|| anyhow::anyhow!("Llm cannot be blank."))?;
        let model = self.model.ok_or_else(|| anyhow::anyhow!("Model cannot be blank"))?;
        let title = self.title.ok_or_else(|| anyhow::anyhow!("Title cannot be blank"))?;
        let prompt = self.prompt.ok_or_else(|| anyhow::anyhow!("Prompt cannot be blank"))?;
        Ok( Chat::new(llm, model, title, self.system, prompt))
    }

}