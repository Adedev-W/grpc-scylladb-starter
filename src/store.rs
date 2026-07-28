use crate::pb;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub(crate) struct ChannelRecord {
    id: u64,
    name: Arc<str>,
    created_at_unix_ms: u64,
}

impl ChannelRecord {
    pub(crate) fn to_proto(&self) -> pb::Channel {
        pb::Channel {
            id: self.id,
            name: self.name.to_string(),
            created_at_unix_ms: self.created_at_unix_ms,
        }
    }
}

#[derive(Default)]
struct ChannelIndex {
    channels: Vec<ChannelRecord>,
    by_id: HashMap<u64, usize>,
}

pub(crate) struct ChannelStore {
    next_id: AtomicU64,
    inner: RwLock<ChannelIndex>,
}

impl ChannelStore {
    pub(crate) fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            inner: RwLock::new(ChannelIndex::default()),
        }
    }

    pub(crate) async fn create(&self, name: String) -> ChannelRecord {
        // Relaxed is enough here; the counter only needs to hand out unique ids.
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let record = ChannelRecord {
            id,
            // Arc keeps the stored name cheap to clone for reads without duplicating the payload.
            name: Arc::from(name),
            created_at_unix_ms: current_unix_ms(),
        };

        let mut guard = self.inner.write().await;
        // Keep the write lock tiny: update the index and append the new row, then release it.
        let index = guard.channels.len();
        guard.by_id.insert(record.id, index);
        guard.channels.push(record.clone());

        record
    }

    pub(crate) async fn get(&self, id: u64) -> Option<ChannelRecord> {
        let guard = self.inner.read().await;
        let index = *guard.by_id.get(&id)?;
        guard.channels.get(index).cloned()
    }

    pub(crate) async fn list(&self, offset: usize, limit: usize) -> (Vec<ChannelRecord>, usize) {
        let guard = self.inner.read().await;
        let total_count = guard.channels.len();

        if offset >= total_count {
            return (Vec::new(), total_count);
        }

        let end = offset.saturating_add(limit).min(total_count);
        // Only copy the requested page so large collections never hit the hot path all at once.
        (guard.channels[offset..end].to_vec(), total_count)
    }
}

fn current_unix_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis().min(u128::from(u64::MAX)) as u64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelStore;

    #[tokio::test]
    async fn create_get_and_list_should_round_trip() {
        let store = ChannelStore::new();

        let first = store.create("alpha".to_string()).await;
        let second = store.create("beta".to_string()).await;

        assert_eq!(first.id, 1);
        assert_eq!(second.id, 2);

        let fetched = store.get(first.id).await.expect("channel exists");
        assert_eq!(fetched.name.as_ref(), "alpha");

        let (page, total_count) = store.list(0, 1).await;
        assert_eq!(total_count, 2);
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].id, first.id);
    }

    #[tokio::test]
    async fn get_missing_should_return_none() {
        let store = ChannelStore::new();

        assert!(store.get(999).await.is_none());
    }
}
