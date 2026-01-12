use agentic_core_rs::capabilities::completion::{message::Message, response};
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
    pub messages: Vec<Message>,
    pub stream: bool
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

    pub fn new(llm: String, model: String, title: String, system: Option<String>, prompt: String, stream: bool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self{id: id, title: title, llm: llm, model: model, system: system, prompt: prompt, messages: Vec::new(), stream: stream}
    }

    // update the user message
    pub fn update_user_message(&mut self, content:String) {
        let message =  Message::create_user_message(&content, None);
        self.messages.push( message);
    }

    // update the assistant message
    pub fn update_assistant_message(&mut self, content: String, response_id: String) {
        let message =  Message::create_assistant_message(&content, Some(response_id));
        self.messages.push(message);
    }
}



#[derive(Deserialize, Debug)]
pub struct ChatConfig {
    pub llm: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
    pub system: Option<String>,
    pub prompt: Option<String>,
    pub stream: bool
}

impl ChatConfig {
    
    pub fn validate(self) -> Result<Chat>{

        let llm = self.llm.ok_or_else(|| anyhow::anyhow!("Llm cannot be blank."))?;
        let model = self.model.ok_or_else(|| anyhow::anyhow!("Model cannot be blank"))?;
        let title = self.title.ok_or_else(|| anyhow::anyhow!("Title cannot be blank"))?;
        let prompt = self.prompt.ok_or_else(|| anyhow::anyhow!("Prompt cannot be blank"))?;
        let stream = self.stream;

        Ok( Chat::new(llm, model, title, self.system, prompt, stream))
    }

}