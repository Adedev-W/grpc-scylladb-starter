use crate::domain::channel::Channel;
use async_trait::async_trait;
use thiserror::Error;

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
    async fn get(&self, id: u64) -> Result<Option<Channel>, RepositoryError>;
    async fn list(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Channel>, usize), RepositoryError>;
}
