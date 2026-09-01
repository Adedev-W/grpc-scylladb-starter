use crate::{
    application::ports::{ChannelRepository, RepositoryError},
    domain::channel::{Channel, ChannelError, validate_name},
};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct ListChannels {
    pub page_token: Vec<u8>,
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

    pub async fn get(&self, id: Uuid) -> Result<Channel, ApplicationError> {
        self.repository
            .get(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn update(&self, id: Uuid, name: String) -> Result<Channel, ApplicationError> {
        let name = validate_name(name)?;
        self.repository
            .update(id, name)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }

    pub async fn list(
        &self,
        query: ListChannels,
    ) -> Result<(Vec<Channel>, Option<Vec<u8>>), ApplicationError> {
        let limit = match usize::try_from(query.limit) {
            Ok(0) | Err(_) => DEFAULT_PAGE_SIZE,
            Ok(value) => value,
        }
        .min(MAX_PAGE_SIZE);
        self.repository
            .list(
                (!query.page_token.is_empty()).then_some(query.page_token),
                limit,
            )
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelService, CreateChannel, ListChannels};
    use crate::{
        application::ports::{ChannelRepository, RepositoryError},
        domain::channel::Channel,
    };
    use async_trait::async_trait;
    use std::sync::Arc;
    use uuid::Uuid;

    struct MockRepository;

    #[async_trait]
    impl ChannelRepository for MockRepository {
        async fn create(&self, name: String) -> Result<Channel, RepositoryError> {
            Ok(Channel {
                id: Uuid::new_v4(),
                name,
                created_at_unix_ms: 1,
            })
        }

        async fn get(&self, _id: Uuid) -> Result<Option<Channel>, RepositoryError> {
            Ok(None)
        }

        async fn update(
            &self,
            _id: Uuid,
            _name: String,
        ) -> Result<Option<Channel>, RepositoryError> {
            Ok(None)
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn list(
            &self,
            page_token: Option<Vec<u8>>,
            limit: usize,
        ) -> Result<(Vec<Channel>, Option<Vec<u8>>), RepositoryError> {
            assert_eq!(page_token, Some(vec![1, 2, 3]));
            assert_eq!(limit, 100);
            Ok((Vec::new(), None))
        }
    }

    #[tokio::test]
    async fn create_validates_before_calling_repository() {
        let service = ChannelService::new(Arc::new(MockRepository));
        let result = service.create(CreateChannel { name: "  ".into() }).await;

        assert!(matches!(
            result,
            Err(super::ApplicationError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn list_uses_default_and_maximum_page_limits() {
        let service = ChannelService::new(Arc::new(MockRepository));
        let result = service
            .list(ListChannels {
                page_token: vec![1, 2, 3],
                limit: 500,
            })
            .await;

        assert!(result.is_ok());
    }
}
