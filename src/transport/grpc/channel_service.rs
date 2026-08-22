use crate::{
    application::{ChannelService, CreateChannel, ListChannels},
    pb,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct GrpcChannelService {
    service: Arc<ChannelService>,
}

impl GrpcChannelService {
    pub fn new(service: ChannelService) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

#[tonic::async_trait]
impl pb::channel_service_server::ChannelService for GrpcChannelService {
    async fn create_channel(
        &self,
        request: Request<pb::CreateChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let channel = self
            .service
            .create(CreateChannel {
                name: request.into_inner().name,
            })
            .await
            .map_err(to_status)?;
        Ok(Response::new(to_proto(channel)))
    }

    async fn get_channel(
        &self,
        request: Request<pb::GetChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let channel = self
            .service
            .get(request.into_inner().id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(to_proto(channel)))
    }

    async fn list_channels(
        &self,
        request: Request<pb::ListChannelsRequest>,
    ) -> Result<Response<pb::ListChannelsResponse>, Status> {
        let request = request.into_inner();
        let (channels, total_count, next_offset) = self
            .service
            .list(ListChannels {
                offset: request.offset,
                limit: request.limit,
            })
            .await
            .map_err(to_status)?;
        Ok(Response::new(pb::ListChannelsResponse {
            channels: channels.into_iter().map(to_proto).collect(),
            next_offset,
            total_count: total_count as u64,
        }))
    }
}

fn to_proto(channel: crate::domain::channel::Channel) -> pb::Channel {
    pb::Channel {
        id: channel.id,
        name: channel.name,
        created_at_unix_ms: channel.created_at_unix_ms,
    }
}

fn to_status(error: crate::application::channel_service::ApplicationError) -> Status {
    match error {
        crate::application::channel_service::ApplicationError::Validation(error) => {
            Status::invalid_argument(error.to_string())
        }
        crate::application::channel_service::ApplicationError::NotFound => {
            Status::not_found("channel not found")
        }
        crate::application::channel_service::ApplicationError::Repository(error) => {
            Status::internal(error.to_string())
        }
    }
}
