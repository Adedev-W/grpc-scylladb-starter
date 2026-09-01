use crate::{
    application::{
        Action, Authorizer, ChannelService, CreateChannel, ListChannels, Principal, Resource,
    },
    pb,
};
use std::sync::Arc;
use tonic::transport::server::{TcpConnectInfo, TlsConnectInfo};
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Clone)]
pub struct GrpcChannelService {
    service: Arc<ChannelService>,
    authorizer: Option<Arc<dyn Authorizer>>,
}

impl GrpcChannelService {
    pub fn new(service: ChannelService) -> Self {
        Self {
            service: Arc::new(service),
            authorizer: None,
        }
    }

    pub fn with_authorizer(service: ChannelService, authorizer: Arc<dyn Authorizer>) -> Self {
        Self {
            service: Arc::new(service),
            authorizer: Some(authorizer),
        }
    }

    async fn authorize<T>(
        &self,
        request: Request<T>,
        action: Action,
        id: Option<String>,
    ) -> Result<T, Status> {
        let Some(authorizer) = &self.authorizer else {
            return Ok(request.into_inner());
        };
        let principal = principal_from_request(&request)?;
        authorizer
            .authorize(
                &principal,
                &Resource {
                    resource_type: "channel".to_string(),
                    id,
                },
                action,
            )
            .await
            .map_err(to_authorization_status)?;
        Ok(request.into_inner())
    }
}

#[tonic::async_trait]
impl pb::channel_service_server::ChannelService for GrpcChannelService {
    async fn create_channel(
        &self,
        request: Request<pb::CreateChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let request = self.authorize(request, Action::Create, None).await?;
        let channel = self
            .service
            .create(CreateChannel { name: request.name })
            .await
            .map_err(to_status)?;
        Ok(Response::new(to_proto(channel)))
    }

    async fn get_channel(
        &self,
        request: Request<pb::GetChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let id = parse_id(&request.get_ref().id)?;
        self.authorize(request, Action::Read, Some(id.to_string()))
            .await?;
        let channel = self.service.get(id).await.map_err(to_status)?;
        Ok(Response::new(to_proto(channel)))
    }

    async fn update_channel(
        &self,
        request: Request<pb::UpdateChannelRequest>,
    ) -> Result<Response<pb::Channel>, Status> {
        let id = parse_id(&request.get_ref().id)?;
        let request = self
            .authorize(request, Action::Update, Some(id.to_string()))
            .await?;
        let channel = self
            .service
            .update(id, request.name)
            .await
            .map_err(to_status)?;
        Ok(Response::new(to_proto(channel)))
    }

    async fn delete_channel(
        &self,
        request: Request<pb::DeleteChannelRequest>,
    ) -> Result<Response<()>, Status> {
        let id = parse_id(&request.get_ref().id)?;
        self.authorize(request, Action::Delete, Some(id.to_string()))
            .await?;
        self.service.delete(id).await.map_err(to_status)?;
        Ok(Response::new(()))
    }

    async fn list_channels(
        &self,
        request: Request<pb::ListChannelsRequest>,
    ) -> Result<Response<pb::ListChannelsResponse>, Status> {
        let request = self.authorize(request, Action::List, None).await?;
        let (channels, next_page_token) = self
            .service
            .list(ListChannels {
                page_token: request.page_token,
                limit: request.limit,
            })
            .await
            .map_err(to_status)?;
        Ok(Response::new(pb::ListChannelsResponse {
            channels: channels.into_iter().map(to_proto).collect(),
            next_page_token: next_page_token.unwrap_or_default(),
        }))
    }
}

fn principal_from_request<T>(request: &Request<T>) -> Result<Principal, Status> {
    let connect_info = request
        .extensions()
        .get::<TlsConnectInfo<TcpConnectInfo>>()
        .ok_or_else(|| Status::unauthenticated("mTLS client certificate is required"))?;
    let certificate = connect_info
        .peer_certs()
        .and_then(|certificates| certificates.first().cloned())
        .ok_or_else(|| Status::unauthenticated("mTLS client certificate is required"))?;
    let (_, certificate) = x509_parser::parse_x509_certificate(certificate.as_ref())
        .map_err(|_| Status::unauthenticated("invalid client certificate"))?;
    let subject = certificate
        .subject()
        .iter_common_name()
        .next()
        .and_then(|name| name.as_str().ok())
        .ok_or_else(|| Status::unauthenticated("client certificate has no common name"))?;
    Ok(Principal {
        subject: subject.to_string(),
    })
}

fn to_proto(channel: crate::domain::channel::Channel) -> pb::Channel {
    pb::Channel {
        id: channel.id.to_string(),
        name: channel.name,
        created_at_unix_ms: channel.created_at_unix_ms,
    }
}

fn parse_id(value: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument("id must be a valid UUID"))
}

fn to_authorization_status(error: crate::application::AuthorizationError) -> Status {
    match error {
        crate::application::AuthorizationError::AccessDenied => {
            Status::permission_denied("access denied")
        }
        crate::application::AuthorizationError::EmptySubject => {
            Status::unauthenticated("client certificate has no subject")
        }
        crate::application::AuthorizationError::Storage(_) => {
            Status::unavailable("authorization service is unavailable")
        }
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
        crate::application::channel_service::ApplicationError::Repository(
            crate::application::ports::RepositoryError::Unavailable(_),
        ) => Status::unavailable("channel repository is unavailable"),
        crate::application::channel_service::ApplicationError::Repository(
            crate::application::ports::RepositoryError::Operation(_),
        ) => Status::internal("channel operation failed"),
    }
}
