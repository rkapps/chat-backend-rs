pub mod memory;
pub mod collection;
pub mod repository;


use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, sync::{LazyLock, Mutex}};
use agentic_core_rs::capabilities::messages::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{Ok, Result};

use crate::handlers::{ChatConfig, ChatRequest};


#[derive(Serialize, Deserialize, Clone)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub llm: String,
    pub model: String,
    pub system: Option<String>,
    pub prompt: String,
    pub messages: Vec<Message>
}


static CHATS_MAP: LazyLock<Mutex<HashMap<String, Chat>>> = LazyLock::new(||{
    let map = HashMap::new();
    Mutex::new(map)
});



pub fn get_chats() ->Result<Vec<Chat>> {
    let map = CHATS_MAP.lock().unwrap();
    let chats = map.clone().into_values().collect();
    Ok(chats)
}


pub fn get_chat_by_id(id: String) -> Result<Chat> {
    let mut map = CHATS_MAP.lock().unwrap();
    let chat = map.get_mut(&id).ok_or_else(|| anyhow::anyhow!("Chat not found: {}", id))?;
    Ok(chat.clone())
}

pub fn add_chat_to_map(config: ChatConfig) -> Result<Chat> {

    let mut map = CHATS_MAP.lock().unwrap();

    let id = Uuid::new_v4().to_string();
    let llm = config.llm.ok_or_else(|| anyhow::anyhow!("Llm cannot be blank."))?;
    let model = config.model.ok_or_else(|| anyhow::anyhow!("Model cannot be blank"))?;
    let title = config.title.ok_or_else(|| anyhow::anyhow!("Title cannot be blank"))?;
    let prompt = config.prompt.ok_or_else(|| anyhow::anyhow!("Prompt cannot be blank"))?;

    let chat = Chat{id: id.clone(), title: title, llm: llm, model: model, system: config.system, prompt: prompt, messages: Vec::new()};
    if map.insert(id.clone(), chat.clone()).is_some() {
        return Err(anyhow::anyhow!(format!("Chat already exists: {:#?}", id)));
    }
    Ok(chat)
}


pub fn add_chatrequest_to_map(request: ChatRequest) -> Result<Chat>{

    let mut map = CHATS_MAP.lock().unwrap();
    let id = request.id;
    let chat = map.get_mut(&id).ok_or_else(|| anyhow::anyhow!("Chat not found: {}", id))?;
    let message = Message::create_user_message(
        &request.prompt,
        None,
    );

    chat.messages.push(message);
    Ok(chat.clone())
}


pub fn add_chatresponse_to_map(id: &String, response_text: String, response_id: Option<String>) -> Result<()>{
    let mut map: std::sync::MutexGuard<'_, HashMap<String, Chat>> = CHATS_MAP.lock().unwrap();
    let chat = map.get_mut(id).ok_or_else(|| anyhow::anyhow!("Chat not found: {}", id))?;
    
    let message = Message::create_assistant_message(
        &response_text,
        response_id
    );

    chat.messages.push(message);
    Ok(())
}

pub async fn load_chatmap_from_disk() -> Result<()>{

    let file_path = Path::new("data/chats.chats.json");
    let file = File::open(file_path).unwrap();
    let chats: Vec<Chat> = serde_json::from_reader(file)?;
    let mut map: std::sync::MutexGuard<'_, HashMap<String, Chat>> = CHATS_MAP.lock().unwrap();
    for chat in chats.clone() {
        map.insert(chat.clone().id, chat);
    }
    // Ok(chats)
    Ok(())
}


pub async fn save_chatmap_to_disk() {

    let chats = get_chats().unwrap();

    let file_path = Path::new("data/chats.chats.json");

    let file = File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    // 4. Serialize the struct to the file
    // serde_json::to_writer takes any io::Write, such as our BufWriter
    serde_json::to_writer_pretty(writer, &chats).unwrap();


}