use crate::domain::channel::Channel;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("repository unavailable: {0}")]
    Unavailable(String),
    #[error("repository operation failed: {0}")]
    Operation(String),
}

#[async_trait]
pub trait ChannelRepository: Send + Sync {
    async fn create(&self, name: String) -> Result<Channel, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Channel>, RepositoryError>;
    async fn update(&self, id: Uuid, name: String) -> Result<Option<Channel>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn list(
        &self,
        page_token: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<(Vec<Channel>, Option<Vec<u8>>), RepositoryError>;
}
