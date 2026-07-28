use crate::{ChannelServiceImpl, pb::channel_service_server::ChannelServiceServer};
use std::net::SocketAddr;
use thiserror::Error;
use tonic::transport::Server;

const DEFAULT_ADDR: &str = "127.0.0.1:50051";
const USAGE: &str = "Usage: server [--addr HOST:PORT]\n";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("{0}")]
    Usage(String),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

pub async fn run_from_env() -> Result<(), ServerError> {
    run_from_args(std::env::args().skip(1)).await
}

pub async fn run_from_args<I>(args: I) -> Result<(), ServerError>
where
    I: IntoIterator<Item = String>,
{
    let addr = parse_args(args.into_iter())?;
    run(addr).await.map_err(ServerError::Transport)
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<SocketAddr, ServerError> {
    let mut addr: SocketAddr = DEFAULT_ADDR.parse()?;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(ServerError::Usage(USAGE.to_string())),
            "-a" | "--addr" => {
                let value = args
                    .next()
                    .ok_or_else(|| ServerError::Usage("--addr requires a value".to_string()))?;
                addr = value.parse()?;
            }
            other => {
                return Err(ServerError::Usage(format!(
                    "unknown argument `{other}`\n\n{USAGE}"
                )));
            }
        }
    }

    Ok(addr)
}

pub async fn run(addr: SocketAddr) -> Result<(), tonic::transport::Error> {
    println!("gRPC server listening on {addr}");

    let service = ChannelServiceServer::new(ChannelServiceImpl::new());
    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}
