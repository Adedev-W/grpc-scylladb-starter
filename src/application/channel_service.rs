use crate::{
    application::ports::{ChannelRepository, RepositoryError},
    domain::channel::{Channel, ChannelError, validate_name},
};
use std::sync::Arc;
use thiserror::Error;

pub const DEFAULT_PAGE_SIZE: usize = 50;
pub const MAX_PAGE_SIZE: usize = 100;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Validation(#[from] ChannelError),
    #[error("channel not found")]
    NotFound,
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Clone)]
pub struct CreateChannel {
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ListChannels {
    pub offset: u64,
    pub limit: u32,
}

#[derive(Clone)]
pub struct ChannelService {
    repository: Arc<dyn ChannelRepository>,
}

impl ChannelService {
    pub fn new(repository: Arc<dyn ChannelRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, command: CreateChannel) -> Result<Channel, ApplicationError> {
        let name = validate_name(command.name)?;
        Ok(self.repository.create(name).await?)
    }

    pub async fn get(&self, id: u64) -> Result<Channel, ApplicationError> {
        if id == 0 {
            return Err(ChannelError::InvalidId.into());
        }
        self.repository
            .get(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn list(
        &self,
        query: ListChannels,
    ) -> Result<(Vec<Channel>, usize, u64), ApplicationError> {
        let offset = usize::try_from(query.offset)
            .map_err(|_| RepositoryError::Operation("offset is too large".into()))?;
        let limit = match usize::try_from(query.limit) {
            Ok(0) | Err(_) => DEFAULT_PAGE_SIZE,
            Ok(value) => value,
        }
        .min(MAX_PAGE_SIZE);
        let (channels, total_count) = self.repository.list(offset, limit).await?;
        let next_offset = if channels.is_empty() {
            total_count as u64
        } else {
            query.offset.saturating_add(channels.len() as u64)
        };
        Ok((channels, total_count, next_offset))
    }
}
