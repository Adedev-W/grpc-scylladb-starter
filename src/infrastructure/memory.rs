use crate::{
    application::ports::{ChannelRepository, RepositoryError},
    domain::channel::Channel,
};
use async_trait::async_trait;
use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct InMemoryChannelRepository {
    channels: Mutex<Vec<Channel>>,
}

#[async_trait]
impl ChannelRepository for InMemoryChannelRepository {
    async fn create(&self, name: String) -> Result<Channel, RepositoryError> {
        let mut channels = self
            .channels
            .lock()
            .map_err(|_| RepositoryError::Operation("lock poisoned".into()))?;
        let channel = Channel {
            id: channels.len() as u64 + 1,
            name,
            created_at_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        channels.push(channel.clone());
        Ok(channel)
    }

    async fn get(&self, id: u64) -> Result<Option<Channel>, RepositoryError> {
        let channels = self
            .channels
            .lock()
            .map_err(|_| RepositoryError::Operation("lock poisoned".into()))?;
        Ok(channels.iter().find(|channel| channel.id == id).cloned())
    }

    async fn update(&self, id: u64, name: String) -> Result<Option<Channel>, RepositoryError> {
        let mut channels = self
            .channels
            .lock()
            .map_err(|_| RepositoryError::Operation("lock poisoned".into()))?;
        let Some(channel) = channels.iter_mut().find(|channel| channel.id == id) else {
            return Ok(None);
        };
        channel.name = name;
        Ok(Some(channel.clone()))
    }

    async fn delete(&self, id: u64) -> Result<bool, RepositoryError> {
        let mut channels = self
            .channels
            .lock()
            .map_err(|_| RepositoryError::Operation("lock poisoned".into()))?;
        let original_len = channels.len();
        channels.retain(|channel| channel.id != id);
        Ok(channels.len() != original_len)
    }

    async fn list(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Channel>, usize), RepositoryError> {
        let channels = self
            .channels
            .lock()
            .map_err(|_| RepositoryError::Operation("lock poisoned".into()))?;
        let total = channels.len();
        Ok((
            channels.iter().skip(offset).take(limit).cloned().collect(),
            total,
        ))
    }
}
