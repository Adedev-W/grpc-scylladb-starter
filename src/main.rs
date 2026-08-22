use grpc_scylladb_starter::{bootstrap, config::AppConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;
    bootstrap::run(config).await?;
    Ok(())
}
