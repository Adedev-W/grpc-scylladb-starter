use crate::{pb, store::ChannelStore};
use std::{convert::TryFrom, sync::Arc};
use tonic::{Request, Response, Status};

const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 100;
const MAX_NAME_BYTES: usize = 128;

#[derive(Clone)]
pub struct ChannelServiceImpl {
    store: Arc<ChannelStore>,
}

impl ChannelServiceImpl {
    pub fn new() -> Self {
        Self {
            store: Arc::new(ChannelStore::new()),
        }
    }
}

impl Default for ChannelServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl pb::channel_service_server::ChannelService for ChannelServiceImpl {
    async fn create_channel(
        &self,
        request: Request<pb::CreateChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let request = request.into_inner();
        let name = validate_name(request.name)?;
        let record = self.store.create(name).await;

        Ok(Response::new(record.to_proto()))
    }

    async fn get_channel(
        &self,
        request: Request<pb::GetChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let id = request.into_inner().id;

        if id == 0 {
            return Err(Status::invalid_argument("id must be greater than zero"));
        }

        let Some(record) = self.store.get(id).await else {
            return Err(Status::not_found("channel not found"));
        };

        Ok(Response::new(record.to_proto()))
    }

    async fn list_channels(
        &self,
        request: Request<pb::ListChannelsRequest>,
    ) -> Result<Response<pb::ListChannelsResponse>, Status> {
        let request = request.into_inner();
        let offset = usize::try_from(request.offset)
            .map_err(|_| Status::invalid_argument("offset is too large"))?;
        let limit = normalize_limit(request.limit);
        let (records, total_count) = self.store.list(offset, limit).await;
        let next_offset = if records.is_empty() {
            total_count as u64
        } else {
            offset.saturating_add(records.len()) as u64
        };

        let channels = records
            .into_iter()
            .map(|record| record.to_proto())
            .collect();

        Ok(Response::new(pb::ListChannelsResponse {
            channels,
            next_offset,
            total_count: total_count as u64,
        }))
    }
}

fn validate_name(name: String) -> Result<String, Status> {
    if name.trim().is_empty() {
        return Err(Status::invalid_argument("name cannot be empty"));
    }

    if name.len() > MAX_NAME_BYTES {
        return Err(Status::invalid_argument("name is too long"));
    }

    Ok(name)
}

fn normalize_limit(limit: u32) -> usize {
    let limit = match usize::try_from(limit) {
        Ok(0) | Err(_) => DEFAULT_PAGE_SIZE,
        Ok(value) => value,
    };

    limit.min(MAX_PAGE_SIZE)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, normalize_limit, validate_name};

    #[test]
    fn validate_name_should_reject_blank_input() {
        assert!(validate_name("   ".to_string()).is_err());
    }

    #[test]
    fn normalize_limit_should_default_and_clamp() {
        assert_eq!(normalize_limit(0), DEFAULT_PAGE_SIZE);
        assert_eq!(normalize_limit(999), MAX_PAGE_SIZE);
    }
}
