use std::{env, net::SocketAddr, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid GRPC_ADDR: {0}")]
    InvalidGrpcAddress(#[from] std::net::AddrParseError),
    #[error("mTLS requires GRPC_TLS_CERT, GRPC_TLS_KEY, and GRPC_TLS_CLIENT_CA")]
    IncompleteTlsConfig,
}

#[derive(Debug, Clone)]
pub struct MtlsConfig {
    pub server_cert: PathBuf,
    pub server_key: PathBuf,
    pub client_ca: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grpc_addr: SocketAddr,
    pub scylla_nodes: Vec<String>,
    pub scylla_keyspace: String,
    pub mtls: Option<MtlsConfig>,
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
        let mtls = parse_mtls_config([
            env::var("GRPC_TLS_CERT").ok(),
            env::var("GRPC_TLS_KEY").ok(),
            env::var("GRPC_TLS_CLIENT_CA").ok(),
        ])?;

        Ok(Self {
            grpc_addr,
            scylla_nodes,
            scylla_keyspace,
            mtls,
        })
    }
}

fn parse_mtls_config(values: [Option<String>; 3]) -> Result<Option<MtlsConfig>, ConfigError> {
    match values {
        [Some(server_cert), Some(server_key), Some(client_ca)] => Ok(Some(MtlsConfig {
            server_cert: server_cert.into(),
            server_key: server_key.into(),
            client_ca: client_ca.into(),
        })),
        [None, None, None] => Ok(None),
        _ => Err(ConfigError::IncompleteTlsConfig),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ConfigError, parse_mtls_config};

    #[test]
    fn defaults_are_local_only() {
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.grpc_addr.port(), 50051);
        assert_eq!(config.scylla_keyspace, "grpc_starter");
    }

    #[test]
    fn partial_mtls_configuration_is_rejected() {
        let result = parse_mtls_config([Some("server.crt".into()), None, None]);

        assert!(matches!(result, Err(ConfigError::IncompleteTlsConfig)));
    }
}
