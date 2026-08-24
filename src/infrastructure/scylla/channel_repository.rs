use crate::{
    application::ports::{ChannelRepository, RepositoryError},
    domain::channel::Channel,
};
use async_trait::async_trait;
use scylla::client::session::Session;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ScyllaChannelRepository {
    session: Arc<Session>,
    next_id: AtomicU64,
}

impl ScyllaChannelRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            next_id: AtomicU64::new(current_time_ms()),
        }
    }

    fn error(error: impl ToString) -> RepositoryError {
        RepositoryError::Operation(error.to_string())
    }
}

#[async_trait]
impl ChannelRepository for ScyllaChannelRepository {
    async fn create(&self, name: String) -> Result<Channel, RepositoryError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let created_at_unix_ms = current_time_ms();
        self.session
            .query_unpaged(
                "INSERT INTO channels (id, name, created_at_unix_ms) VALUES (?, ?, ?)",
                (id as i64, name.as_str(), created_at_unix_ms as i64),
            )
            .await
            .map_err(Self::error)?;
        Ok(Channel {
            id,
            name,
            created_at_unix_ms,
        })
    }

    async fn get(&self, id: u64) -> Result<Option<Channel>, RepositoryError> {
        let result = self
            .session
            .query_unpaged(
                "SELECT id, name, created_at_unix_ms FROM channels WHERE id = ?",
                (id as i64,),
            )
            .await
            .map_err(Self::error)?
            .into_rows_result()
            .map_err(Self::error)?;
        let mut rows = result.rows::<(i64, String, i64)>().map_err(Self::error)?;
        rows.next()
            .transpose()
            .map_err(Self::error)
            .map(|channel| channel.map(to_domain))
    }

    async fn update(&self, id: u64, name: String) -> Result<Option<Channel>, RepositoryError> {
        if self.get(id).await?.is_none() {
            return Ok(None);
        }
        self.session
            .query_unpaged(
                "UPDATE channels SET name = ? WHERE id = ?",
                (name, id as i64),
            )
            .await
            .map_err(Self::error)?;
        self.get(id).await
    }

    async fn delete(&self, id: u64) -> Result<bool, RepositoryError> {
        if self.get(id).await?.is_none() {
            return Ok(false);
        }
        self.session
            .query_unpaged("DELETE FROM channels WHERE id = ?", (id as i64,))
            .await
            .map_err(Self::error)?;
        Ok(true)
    }

    async fn list(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Channel>, usize), RepositoryError> {
        let result = self
            .session
            .query_unpaged(
                "SELECT id, name, created_at_unix_ms FROM channels",
                &[] as &[i32],
            )
            .await
            .map_err(Self::error)?
            .into_rows_result()
            .map_err(Self::error)?;
        let channels = result
            .rows::<(i64, String, i64)>()
            .map_err(Self::error)?
            .map(|row| row.map(to_domain).map_err(Self::error))
            .collect::<Result<Vec<_>, _>>()?;
        let total_count = channels.len();
        let page = channels.into_iter().skip(offset).take(limit).collect();
        Ok((page, total_count))
    }
}

fn to_domain(row: (i64, String, i64)) -> Channel {
    Channel {
        id: row.0 as u64,
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
