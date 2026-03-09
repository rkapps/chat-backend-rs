use std::{collections::HashMap, sync::Arc};

use crate::{
    chat::{
        dto::{ChatRequest, ChatResponse, ChatStreamingMessage},
        model::{Chat, ChatConfig},
        storage::ChatStorage,
    },
};
use agentic_core::{
    agent::{completion::Agent, service::LlmProvider},
    capabilities::{
        client::completion::CompletionStreamResponse,
        completion::{
            message::Message, response::CompletionResponseContent,
        },
    },
};
use anyhow::Result;
use storage_core::core::Repository;
use tracing::debug;

#[derive(Debug)]
pub struct ChatService {
    storage:ChatStorage,
    pub agents: HashMap<String, Arc<Agent>>,
    pub providers: Vec<LlmProvider>
}

impl ChatService {
    pub fn new(storage: ChatStorage) -> ChatService {
        Self {
            storage,
            agents: HashMap::new(),
            providers: Vec::new()
        }
    }

    pub fn add_llm_provider(&mut self, providers: Vec<LlmProvider>) {
        self.providers = providers
    }

    pub fn add_agent(&mut self, agent: Agent) {
        let key = self.key(&agent.llm, &agent.model);
        self.agents.insert(key, Arc::new(agent));
    }

    pub fn get_agent(&self, llm: &str, model: &str) -> Arc<Agent> {
        let key = self.key(llm, model);
        self.agents.get(&key).unwrap().clone()
    }

    fn key(&self, llm: &str, model: &str) -> String {
        format!("{}:{}", llm, model)
    }

    pub fn get_llm_providers(&self) -> Vec<LlmProvider> {
        self.providers.clone()
    }

    pub async fn create_chat(&self, config: ChatConfig) -> Result<Chat> {
        let chat = config.validate().map_err(|e| anyhow::anyhow!(e))?;

        match self.storage.chats().await {
            Ok(repo) => {
                let mut repo = repo.lock().await;
                let _ = repo.insert(chat.clone()).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error getting chats: {}", e));
            }
        }
        // // Lock the storage to get mutable access
        // let mut storage_guard = self.storage.lock().await;
        // storage_guard
        //     .create_chat(chat.clone())
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;
        Ok(chat)
    }

    pub async fn get_all_chats(&self) -> Result<Vec<Chat>> {

        match self.storage.chats().await {
            Ok(repo) => {
                let mut repo = repo.lock().await;
                repo.find_all().await
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error getting chats: {}", e));
            }
        }
        // let mut storage_guard = self.storage.lock().await;
        // let chats = storage_guard
        //     .get_all_chats()
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;
        // Ok(chats)
    }

    pub async fn get_chat_by_id(&self, id: String) -> Result<Chat> {

        match self.storage.chats().await {
            Ok(repo) => {
                let mut repo = repo.lock().await;
                repo.find_by_id(id).await
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error getting chats: {}", e));
            }
        }

        // let mut storage_guard = self.storage.lock().await;
        // let chat = storage_guard
        //     .get_chat(id.clone())
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;
        // Ok(chat)
    }

    pub async fn delete_chat(&self, id: String) -> Result<()> {

        let chat = self.get_chat_by_id(id).await?;
        match self.storage.chats().await {
            Ok(repo) => {
                let mut repo = repo.lock().await;
                repo.delete(chat).await
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error getting chats: {}", e));
            }
        }        
        // let mut storage_guard = self.storage.lock().await;
        // storage_guard
        //     .delete_chat(id)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;
        // Ok(())
    }

    pub async fn update_chat(&self, chat: Chat) -> Result<()> {

        match self.storage.chats().await {
            Ok(repo) => {
                let mut repo = repo.lock().await;
                repo.update(chat).await
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error getting chats: {}", e));
            }
        }        
        // let mut storage_guard = self.storage.lock().await;
        // storage_guard
        //     .delete_chat(id)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;
        // Ok(())
    }


    pub async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse> {

        let mut chat = self.get_chat_by_id(request.clone().id).await?;

        // let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        // let mut chat = storage_guard
        //     .get_chat(request.clone().id)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;

        let agent = self.get_agent(&chat.llm, &chat.model);
        let id = chat.clone().id;
        chat.update_user_message(request.prompt);

        let messages = self.create_completion_messages(&chat);
        // On error, Send an error response
        let cresponse = agent
            .complete(&chat.system, &messages)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Completion Response error {:?}", e)))?;

        debug!("Completion Response: {:#?}", cresponse);
        //Create the chat response message and add it the chat.
        let mut rcontent = String::new();
        let response = cresponse.clone();
        for content in response.contents {
            if let CompletionResponseContent::Text(val) = content {
                // println!("The text is: {}", val);
                rcontent = val.to_string();
                chat.update_assistant_message(val.to_string(), response.response_id.clone());
            }
        }

        self.update_chat(chat).await?;
        // storage_guard
        //     .update_chat(chat)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;

        //Create the chat response
        let response = ChatResponse {
            id: id,
            role: "assistant".to_string(),
            content: Some(rcontent),
            response_id: Some(response.response_id),
        };
        debug!("Chat Request: {:#?}", response);

        Ok(response)
    }

    pub async fn chat_completion_streaming(
        &self,
        request: ChatRequest,
    ) -> Result<CompletionStreamResponse> {
        debug!("Chat request: {:?}", request);

        let mut chat = self.get_chat_by_id(request.clone().id).await?;
        // let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        // let mut chat = storage_guard
        //     .get_chat(request.clone().id)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;

        chat.update_user_message(request.prompt);
        let messages = self.create_completion_messages(&chat);

        let agent = self.get_agent(&chat.llm, &chat.model);
        // let crequest = self.create_completion_request(chat, agent.temperature, agent.max_tokens);
        let stream = agent.complete_with_stream(&chat.system, &messages).await?;
        Ok(stream)
    }

    pub async fn save_streaming_message(&self, request: ChatStreamingMessage) -> Result<()> {
        let mut chat = self.get_chat_by_id(request.clone().id).await?;

        // let mut storage_guard: tokio::sync::MutexGuard<'_, ChatStorage> = self.storage.lock().await;
        // let mut chat = storage_guard
        //     .get_chat(request.clone().id)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;
        chat.update_user_message(request.user_content);
        chat.update_assistant_message(request.assistant_content, request.response_id);

        self.update_chat(chat).await?;
        // storage_guard
        //     .update_chat(chat)
        //     .await
        //     .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
    }

    // fn create_completion_request(
    //     &self,
    //     chat: Chat,
    //     temperature: f32,
    //     max_tokens: i32,
    // ) -> CompletionRequest {
    //     //Create the completion request

    //     let mut nmessages = Vec::new();
    //     for message in chat.messages {
    //         if message.role == "user".to_string() {
    //             let nmessage = Message::User {
    //                 content: message.content,
    //                 response_id: Some(message.response_id),
    //             };
    //             nmessages.push(nmessage);
    //         } else {
    //             let nmessage = Message::Assistant {
    //                 content: message.content,
    //                 response_id: Some(message.response_id),
    //             };
    //             nmessages.push(nmessage);
    //         }
    //     }
    //     let crequest = CompletionRequest {
    //         model: chat.model,
    //         system: chat.system,
    //         messages: nmessages,
    //         temperature: temperature,
    //         max_tokens: max_tokens,
    //         stream: chat.stream,
    //         definitions: Vec::new(),
    //     };

    //     crequest
    // }

    fn create_completion_messages(&self, chat: &Chat) -> Vec<Message>{
        let mut nmessages = Vec::new();
        for message in chat.clone().messages {
            if message.role == "user".to_string() {
                let nmessage = Message::User {
                    content: message.content,
                    response_id: Some(message.response_id),
                };
                nmessages.push(nmessage);
            } else {
                let nmessage = Message::Assistant {
                    content: message.content,
                    response_id: Some(message.response_id),
                };
                nmessages.push(nmessage);
            }
        }
        nmessages
    }
}
