use std::sync::Arc;

use crate::chat::{
    dto::{ChatRequest, ChatResponse, ChatStreamingMessage},
    model::{Chat, ChatConfig},
    storage::ChatStorage,
};
use agentic_core::{
    agent::service::AgentService,
    capabilities::{
        client::completion::CompletionStreamResponse,
        completion::{message::MessageRole, request::CompletionRequest},
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
        agent_service: Arc<AgentService>,
    ) -> Result<ChatResponse> {
        let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        let mut chat = storage_guard
            .get_chat(request.clone().id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // let agent = self.clone().get_chat_agent(&chat, agent_service)?;
        let agent = agent_service.get_chat_agent(&chat.llm)?;

        let id = chat.clone().id;
        chat.update_user_message(request.prompt);

        let crequest =
            self.create_completion_request(chat.clone(), agent.temperature, agent.max_tokens);

        // On error, Send an error response
        let cresponse = agent
            .complete(crequest)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Completion Response error {:?}", e)))?;

        //Create the chat response message and add it the chat.
        let response = cresponse.clone();
        chat.update_assistant_message(cresponse.content, cresponse.response_id);

        storage_guard
            .update_chat(chat)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        //Create the chat response
        let response = ChatResponse {
            id: id,
            role: MessageRole::Assistant.as_str().to_string(),
            content: Some(response.content),
            response_id: Some(response.response_id),
        };
        debug!("Chat Request: {:#?}", response);

        Ok(response)
    }

    pub async fn chat_completion_streaming(
        &self,
        request: ChatRequest,
        agent_service: Arc<AgentService>,
    ) -> Result<CompletionStreamResponse> {
        debug!("Chat request: {:?}", request);

        let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        let mut chat = storage_guard
            .get_chat(request.clone().id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let agent = agent_service.get_chat_agent(&chat.llm)?;

        chat.update_user_message(request.prompt);

        let crequest = self.create_completion_request(chat, agent.temperature, agent.max_tokens);
        agent.complete_with_stream(crequest).await
    }

    pub async fn save_streaming_message(&self, request: ChatStreamingMessage) -> Result<()> {
        let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        let mut chat = storage_guard
            .get_chat(request.clone().id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        chat.update_user_message(request.user_content);
        chat.update_assistant_message(request.assistant_content, request.response_id);
        storage_guard
            .update_chat(chat)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
    }


    fn create_completion_request(
        &self,
        chat: Chat,
        temperature: f32,
        max_tokens: i32,
    ) -> CompletionRequest {
        //Create the completion request
        let crequest = CompletionRequest {
            model: chat.model,
            system: chat.system,
            messages: chat.messages,
            temperature: temperature,
            max_tokens: max_tokens,
            stream: chat.stream,
        };

        crequest
    }
}
