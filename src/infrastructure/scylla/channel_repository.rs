use crate::{
    application::ports::{ChannelRepository, RepositoryError},
    domain::channel::Channel,
};
use async_trait::async_trait;
use scylla::client::session::Session;
use scylla::response::{PagingState, PagingStateResponse};
use scylla::statement::unprepared::Statement;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct ScyllaChannelRepository {
    session: Arc<Session>,
}

impl ScyllaChannelRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    fn error(error: impl ToString) -> RepositoryError {
        RepositoryError::Operation(error.to_string())
    }
}

#[async_trait]
impl ChannelRepository for ScyllaChannelRepository {
    async fn create(&self, name: String) -> Result<Channel, RepositoryError> {
        let id = Uuid::new_v4();
        let created_at_unix_ms = current_time_ms();
        self.session
            .query_unpaged(
                "INSERT INTO channels (id, name, created_at_unix_ms) VALUES (?, ?, ?)",
                (id, name.as_str(), created_at_unix_ms as i64),
            )
            .await
            .map_err(Self::error)?;
        Ok(Channel {
            id,
            name,
            created_at_unix_ms,
        })
    }

    async fn get(&self, id: Uuid) -> Result<Option<Channel>, RepositoryError> {
        let result = self
            .session
            .query_unpaged(
                "SELECT id, name, created_at_unix_ms FROM channels WHERE id = ?",
                (id,),
            )
            .await
            .map_err(Self::error)?
            .into_rows_result()
            .map_err(Self::error)?;
        let mut rows = result.rows::<(Uuid, String, i64)>().map_err(Self::error)?;
        rows.next()
            .transpose()
            .map_err(Self::error)
            .map(|channel| channel.map(to_domain))
    }

    async fn update(&self, id: Uuid, name: String) -> Result<Option<Channel>, RepositoryError> {
        if self.get(id).await?.is_none() {
            return Ok(None);
        }
        self.session
            .query_unpaged("UPDATE channels SET name = ? WHERE id = ?", (name, id))
            .await
            .map_err(Self::error)?;
        self.get(id).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        if self.get(id).await?.is_none() {
            return Ok(false);
        }
        self.session
            .query_unpaged("DELETE FROM channels WHERE id = ?", (id,))
            .await
            .map_err(Self::error)?;
        Ok(true)
    }

    async fn list(
        &self,
        page_token: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<(Vec<Channel>, Option<Vec<u8>>), RepositoryError> {
        let statement = Statement::new("SELECT id, name, created_at_unix_ms FROM channels")
            .with_page_size(limit as i32);
        let paging_state = page_token
            .map(PagingState::new_from_raw_bytes)
            .unwrap_or_default();
        let (result, paging_response) = self
            .session
            .query_single_page(statement, &[] as &[i32], paging_state)
            .await
            .map_err(Self::error)?;
        let channels = result
            .into_rows_result()
            .map_err(Self::error)?
            .rows::<(Uuid, String, i64)>()
            .map_err(Self::error)?
            .map(|row| row.map(to_domain).map_err(Self::error))
            .collect::<Result<Vec<_>, _>>()?;
        let next_page_token = match paging_response {
            PagingStateResponse::HasMorePages { state } => {
                state.as_bytes_slice().map(|bytes| bytes.as_ref().to_vec())
            }
            PagingStateResponse::NoMorePages => None,
        };
        Ok((channels, next_page_token))
    }
}

fn to_domain(row: (Uuid, String, i64)) -> Channel {
    Channel {
        id: row.0,
        name: row.1,
        created_at_unix_ms: row.2 as u64,
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
