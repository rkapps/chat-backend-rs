use std::sync::Arc;

use crate::{
    agents::service::AgentService,
    chat::{
        dto::{ChatRequest, ChatResponse},
        model::{Chat, ChatConfig},
        storage::ChatStorage,
    },
};
use agentic_core_rs::{
    capabilities::{
        completion::CompletionRequest,
        messages::{Message, MessageRole},
    },
};
use anyhow::Result;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Clone)]
pub struct ChatService {
    storage: Arc<Mutex<ChatStorage>>,
}

impl ChatService {
    pub fn new(storage: Arc<Mutex<ChatStorage>>) -> ChatService {
        Self { storage }
    }

    pub async fn create_chat(&self, config: ChatConfig) -> Result<Chat> {
        let chat = config.validate().map_err(|e| anyhow::anyhow!(e))?;

        // Lock the storage to get mutable access
        let mut storage_guard = self.storage.lock().await;
        storage_guard
            .create_chat(chat.clone())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(chat)
    }

    pub async fn get_all_chats(&self) -> Result<Vec<Chat>> {
        let mut storage_guard = self.storage.lock().await;
        let chats = storage_guard
            .get_all_chats()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(chats)
    }

    pub async fn get_chat_by_id(&self, id: String) -> Result<Chat> {
        let mut storage_guard = self.storage.lock().await;
        let chat = storage_guard
            .get_chat(id.clone())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(chat)
    }

    pub async fn delete_chat(&self, id: String) -> Result<()> {
        let mut storage_guard = self.storage.lock().await;
        storage_guard
            .delete_chat(id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    pub async fn chat_completion(
        &self,
        request: ChatRequest,
        agent_service: AgentService,
    ) -> Result<ChatResponse> {


        let mut storage_guard = self.storage.lock().await;
        let chat = storage_guard
            .get_chat(request.clone().id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let agent = agent_service
            .clone()
            .get_chat_agent(chat.clone().llm, chat.clone().model)
            .map_err(|_e| {
                anyhow::anyhow!(format!(
                    "Agent error for {:?}/{:?}",
                    chat.clone().llm,
                    chat.clone().model
                ))
            })?;

        let id = chat.clone().id;
        let mut nchat = chat.clone();
        // add the chat request to the chat messages.
        let cmessage = Message::create_user_message(&request.prompt, None);
        nchat.messages.push(cmessage);

        //Create the completion request
        let crequest = CompletionRequest {
            model: agent.config.client.model().to_string(),
            system: nchat.system.clone(),
            messages: nchat.messages.clone(),
            max_tokens: agent.max_tokens(),
            temperature: agent.temperature(),
        };

        // On error, Send an error response
        let cresponse = agent
            .complete(crequest)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Completion Response error {:?}", e)))?;

        //Create the chat response message and add it the chat.
        let content = cresponse.content;
        let response_id = Some(cresponse.id);
        let message = Message::create_assistant_message(&content, response_id.clone());
        nchat.messages.push(message);

        storage_guard
            .update_chat(nchat)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        //Create the chat response
        let response = ChatResponse {
            id: id,
            role: MessageRole::Assistant.as_str().to_string(),
            content: Some(content.clone()),
            response_id: response_id,
        };
        debug!("Chat Request: {:#?}", response);

        Ok(response)
    }
}
