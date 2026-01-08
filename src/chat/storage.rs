use crate::chat::model::Chat;
use anyhow::Result;
use storage_core::{core::Repository, fs::database::FsDatabase};

pub struct ChatStorage {
    db: FsDatabase,
    collection_name: String,
}

impl ChatStorage {
    pub fn new(name: String, file_path: String, collection_name: String) -> Self {
        ChatStorage {
            db: FsDatabase::new(name, file_path),
            collection_name,
        }
    }

    pub async fn create_chat(&mut self, chat: Chat) -> Result<()> {
        // let repo: &mut dyn Repository<String, Chat> = self
        //     .db
        //     .collection(self.collection_name.clone())?;
        // .map_err(|e| anyhow::anyhow!(format!("Error creating chat: {:?}", e)))?;
        let repo = self.db.collection(self.collection_name.clone())?;
        repo.insert(chat).await?;
        // .await
        // .map_err(|e| anyhow::anyhow!(format!("Error creating chat: {:?}", e)))?;
        Ok(())
    }

    pub async fn get_all_chats(&mut self) -> Result<Vec<Chat>> {
        let repo = self.db.collection(self.collection_name.clone())?;
        Ok(repo.find_all().await)
    }

    pub async fn get_chat(&mut self, id: String) -> Result<Chat> {
        let repo = self.db.collection(self.collection_name.clone())?;
        let chat = repo
            .find_by_id(id.clone())
            .await
            .ok_or_else(|| anyhow::anyhow!("Chat not found: {}", id))?;
        Ok(chat)
    }

    pub async fn delete_chat(&mut self, id: String) -> Result<()> {
        let repo: &mut dyn Repository<String, Chat> =
            self.db.collection(self.collection_name.clone())?;
        repo.delete(id.clone()).await?;
        Ok(())
    }

    pub async fn update_chat(&mut self, chat: Chat) -> Result<()> {
        let repo = self.db.collection(self.collection_name.clone())?;
        repo.update(chat).await?;
        // .await
        // .map_err(|e| anyhow::anyhow!(format!("Error creating chat: {:?}", e)))?;
        Ok(())
    }
    
    
}
