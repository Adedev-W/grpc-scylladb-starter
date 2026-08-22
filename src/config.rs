use std::{env, net::SocketAddr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid GRPC_ADDR: {0}")]
    InvalidGrpcAddress(#[from] std::net::AddrParseError),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grpc_addr: SocketAddr,
    pub scylla_nodes: Vec<String>,
    pub scylla_keyspace: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let grpc_addr = env::var("GRPC_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:50051".to_string())
            .parse()?;
        let scylla_nodes = env::var("SCYLLA_NODES")
            .unwrap_or_else(|_| "127.0.0.1:9042".to_string())
            .split(',')
            .map(str::trim)
            .filter(|node| !node.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        let scylla_keyspace =
            env::var("SCYLLA_KEYSPACE").unwrap_or_else(|_| "grpc_starter".to_string());

        Ok(Self {
            grpc_addr,
            scylla_nodes,
            scylla_keyspace,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn defaults_are_local_only() {
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.grpc_addr.port(), 50051);
        assert_eq!(config.scylla_keyspace, "grpc_starter");
    }
}
