use crate::{
    application::ChannelService, config::AppConfig,
    infrastructure::memory::InMemoryChannelRepository,
    pb::channel_service_server::ChannelServiceServer,
    transport::grpc::channel_service::GrpcChannelService,
};
use std::sync::Arc;
use thiserror::Error;
use tonic::transport::Server;

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

pub async fn run(config: AppConfig) -> Result<(), BootstrapError> {
    let repository = Arc::new(InMemoryChannelRepository::default());
    let channel_service = ChannelService::new(repository);
    let grpc_service = GrpcChannelService::new(channel_service);

    log_server_start(&config);

    Server::builder()
        .add_service(ChannelServiceServer::new(grpc_service))
        .serve_with_shutdown(config.grpc_addr, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    Ok(())
}

fn log_server_start(config: &AppConfig) {
    eprintln!("gRPC server listening on {}", config.grpc_addr);
}
