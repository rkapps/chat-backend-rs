use std::sync::Arc;

use crate::chat::model::Chat;
use anyhow::Result;
use storage_core::fs::{database::FsDatabase, repository::FsRepository};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ChatStorage {
    db: FsDatabase,
    collection_name: String,
}

impl ChatStorage {
    pub async fn new(name: String, file_path: String, collection_name: String) -> Result<Self> {
        let mut db = FsDatabase::new(name, file_path).await?;
        db.register_collection::<String, Chat>(collection_name.clone())
            .await?;
        Ok(ChatStorage {
            db,
            collection_name,
        })
    }

    pub async fn chats(&self) -> Result<Arc<Mutex<FsRepository<String, Chat>>>> {
        self.db
            .collection::<String, Chat>(self.collection_name.to_string())
            .await
    }


    // pub async fn create_chat(&mut self, chat: Chat) -> Result<()> {
    //     let repo = self.db.collection(self.collection_name.clone()).await?;

    //     let mut repo = repo.lock().await;
    //     repo.insert(chat).await?;

    //     // repo.insert(chat).await?;
    //     Ok(())
    // }

    // pub async fn get_all_chats(&mut self) -> Result<Vec<Chat>> {
    //     let repo = self.db.collection(self.collection_name.clone()).await?;
    //     let mut repo = repo.lock().await;
    //     Ok(repo.find_all().await?)
    // }

    // pub async fn get_chat(&mut self, id: String) -> Result<Chat> {
    //     let repo = self.db.collection::<String, Chat>(self.collection_name.clone()).await?;
    //     let mut repo = repo.lock().await;
    //     let chat = repo
    //         .find_by_id(id.clone())
    //         .await?;
    //     Ok(chat)
    // }

    // pub async fn delete_chat(&mut self, id: String) -> Result<()> {
    //     let repo = self.db.collection::<String, Chat>(self.collection_name.clone()).await?;
    //     let mut repo = repo.lock().await;

    //     let chat = repo
    //         .find_by_id(id.clone())
    //         .await?;
    //     repo.delete(chat).await?;
    //     Ok(())
    // }

    // pub async fn update_chat(&mut self, chat: Chat) -> Result<()> {
    //     let repo = self.db.collection::<String, Chat>(self.collection_name.clone()).await?;
    //     let mut repo = repo.lock().await;
    //     repo.update(chat).await?;
    //     Ok(())
    // }
}
