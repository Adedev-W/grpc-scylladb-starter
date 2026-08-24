use crate::{
    application::ChannelService,
    config::AppConfig,
    infrastructure::scylla::{ScyllaAuthorizer, ScyllaChannelRepository, connect},
    pb::channel_service_server::ChannelServiceServer,
    transport::grpc::channel_service::GrpcChannelService,
};
use std::sync::Arc;
use thiserror::Error;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("scylla connection failed: {0}")]
    Scylla(String),
    #[error("mTLS certificate could not be read: {0}")]
    MtlsIo(#[from] std::io::Error),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

pub async fn run(config: AppConfig) -> Result<(), BootstrapError> {
    let session = connect(&config).await.map_err(BootstrapError::Scylla)?;
    let session = Arc::new(session);
    let authorizer = Arc::new(ScyllaAuthorizer::new(Arc::clone(&session)));
    let repository = Arc::new(ScyllaChannelRepository::new(session));
    let channel_service = ChannelService::new(repository);
    let grpc_service = match &config.mtls {
        Some(_) => GrpcChannelService::with_authorizer(channel_service, authorizer),
        None => GrpcChannelService::new(channel_service),
    };

    log_server_start(&config);

    let mut server = Server::builder();
    if let Some(mtls) = &config.mtls {
        let server_cert = std::fs::read(&mtls.server_cert)?;
        let server_key = std::fs::read(&mtls.server_key)?;
        let client_ca = std::fs::read(&mtls.client_ca)?;
        let tls = ServerTlsConfig::new()
            .identity(Identity::from_pem(server_cert, server_key))
            .client_ca_root(Certificate::from_pem(client_ca));
        server = server.tls_config(tls)?;
    }

    server
        .add_service(ChannelServiceServer::new(grpc_service))
        .serve_with_shutdown(config.grpc_addr, async {
            let _ = tokio::signal::ctrl_c().await; // graceful shutdown on Ctrl+C
        })
        .await?;

    Ok(())
}

fn log_server_start(config: &AppConfig) {
    eprintln!("gRPC server listening on {}", config.grpc_addr);
}
