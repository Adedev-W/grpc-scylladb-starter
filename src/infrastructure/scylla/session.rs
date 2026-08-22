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
        .query_unpaged(
            format!(
                "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'NetworkTopologyStrategy', 'datacenter1': 1}}",
                config.scylla_keyspace
            ),
            &[] as &[i32],
        )
        .await
        .map_err(|error| error.to_string())?;
    session
        .query_unpaged(
            format!(
                "ALTER KEYSPACE {} WITH replication = {{'class': 'NetworkTopologyStrategy', 'datacenter1': 1}}",
                config.scylla_keyspace
            ),
            &[] as &[i32],
        )
        .await
        .map_err(|error| error.to_string())?;
    session
        .use_keyspace(config.scylla_keyspace.clone(), false)
        .await
        .map_err(|error| error.to_string())?;
    session
        .query_unpaged(
            "CREATE TABLE IF NOT EXISTS channels (id bigint PRIMARY KEY, name text, created_at_unix_ms bigint)",
            &[] as &[i32],
        )
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
