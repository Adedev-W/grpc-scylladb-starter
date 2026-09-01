use grpc_scylladb_starter::pb::channel_service_client::ChannelServiceClient;
use std::{env, fs};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

pub async fn connect_client(
    subject: Option<&str>,
) -> Result<ChannelServiceClient<Channel>, Box<dyn std::error::Error>> {
    let endpoint_url =
        env::var("TEST_GRPC_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:50051".into());
    let mut endpoint = Endpoint::from_shared(endpoint_url.clone())?;

    if endpoint_url.starts_with("https://") {
        let subject = subject.unwrap_or("admin.example");
        let ca_path = env::var("TEST_GRPC_CA").unwrap_or_else(|_| "certs/dev/ca.crt".into());
        let cert_path = env::var("TEST_GRPC_CLIENT_CERT")
            .unwrap_or_else(|_| format!("certs/dev/{subject}.crt"));
        let key_path =
            env::var("TEST_GRPC_CLIENT_KEY").unwrap_or_else(|_| format!("certs/dev/{subject}.key"));
        let tls = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(fs::read(ca_path)?))
            .identity(Identity::from_pem(
                fs::read(cert_path)?,
                fs::read(key_path)?,
            ))
            .domain_name("localhost");
        endpoint = endpoint.tls_config(tls)?;
    }

    Ok(ChannelServiceClient::new(endpoint.connect().await?))
}

#[allow(dead_code)]
pub async fn connect_without_client_certificate() -> Result<Channel, Box<dyn std::error::Error>> {
    let endpoint_url =
        env::var("TEST_GRPC_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:50051".into());
    let endpoint = Endpoint::from_shared(endpoint_url)?;
    let ca_path = env::var("TEST_GRPC_CA").unwrap_or_else(|_| "certs/dev/ca.crt".into());
    let tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(fs::read(ca_path)?));
    Ok(endpoint.tls_config(tls)?.connect().await?)
}
