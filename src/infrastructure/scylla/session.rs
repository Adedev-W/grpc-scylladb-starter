use crate::config::AppConfig;
use scylla::client::{session::Session, session_builder::SessionBuilder};

pub async fn connect(config: &AppConfig) -> Result<Session, String> {
    if config.scylla_nodes.is_empty() {
        return Err("SCYLLA_NODES must contain at least one node".to_string());
    }
    if !is_cql_identifier(&config.scylla_keyspace) {
        return Err(
            "SCYLLA_KEYSPACE must contain only letters, numbers, and underscores".to_string(),
        );
    }

    let session = SessionBuilder::new()
        .known_nodes(&config.scylla_nodes)
        .build()
        .await
        .map_err(|error| error.to_string())?;
    session
        .use_keyspace(config.scylla_keyspace.clone(), false)
        .await
        .map_err(|error| error.to_string())?;
    Ok(session)
}

fn is_cql_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
