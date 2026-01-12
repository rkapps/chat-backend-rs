use std::sync::Arc;

use crate::chat::{
    dto::{ChatRequest, ChatResponse},
    model::{Chat, ChatConfig},
    storage::ChatStorage,
};
use agentic_core_rs::{
    agent::{completion::Agent, service::AgentService},
    capabilities::{
        client::completion::CompletionStreamResponse,
        completion::{
            message::{Message, MessageRole},
            request::CompletionRequest,
        },
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

        let agent = self.clone().get_chat_agent(&chat, agent_service)?;

        let id = chat.clone().id;
        // let cmessage = Message::create_user_message(&request.prompt, None);
        // chat.messages.push(cmessage);
        chat.update_user_message(request.prompt);

        let crequest =
            self.create_completion_request(chat.clone(), agent.temperature, agent.max_tokens);

        // On error, Send an error response
        let cresponse = agent
            .complete(crequest)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Completion Response error {:?}", e)))?;

        // chat.messages.push(cmessage);

        //Create the chat response message and add it the chat.
        let response = cresponse.clone();
        // let response_id = Some(cresponse.id.clone());
        // let message = Message::create_assistant_message(&content, response_id.clone());
        // chat.messages.push(message);
        chat.update_assistant_message(cresponse.content, cresponse.id);
        // let mut messages = Vec::new();
        // messages.push(cmessage);
        // messages.push(message);
        // self.update_chat_messages(chat, messages).await?;

        storage_guard
            .update_chat(chat)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        //Create the chat response
        let response = ChatResponse {
            id: id,
            role: MessageRole::Assistant.as_str().to_string(),
            content: Some(response.content),
            response_id: Some(response.id),
        };
        debug!("Chat Request: {:#?}", response);

        Ok(response)
    }

    pub async fn chat_completion_streaming(
        &self,
        request: ChatRequest,
        agent_service: Arc<AgentService>,
    ) -> Result<CompletionStreamResponse> {
        // let crequest = self.create_completion_request(request, agent_service);
        debug!("Chat request: {:?}", request);

        let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        let mut chat = storage_guard
            .get_chat(request.clone().id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let agent = self.clone().get_chat_agent(&chat, agent_service)?;

        // let cmessage = Message::create_user_message(&request.prompt, None);
        // chat.messages.push(cmessage);
        chat.update_user_message(request.prompt);

        // let crequest = CompletionRequest {
        //     model: chat.model,
        //     system: chat.system,
        //     messages: chat.messages,
        //     temperature: agent.temperature,
        //     max_tokens: agent.max_tokens,
        //     stream: chat.stream,
        // };
        let crequest =
            self.create_completion_request(chat, agent.temperature, agent.max_tokens);
        debug!("Completion request: {:?}", crequest);
        debug!("Chat request1111: {:?}", request.id);
        agent.complete_with_stream(crequest).await


    }

    fn get_chat_agent(self, chat: &Chat, agent_service: Arc<AgentService>) -> Result<Arc<Agent>> {
        agent_service
            .clone()
            .get_chat_agent(&chat.llm)
            .map_err(|_e| {
                anyhow::anyhow!(format!("Agent error for {:?}/{:?}", chat.llm, chat.model))
            })
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
