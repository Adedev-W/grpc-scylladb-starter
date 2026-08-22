use thiserror::Error;

pub const MAX_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: u64,
    pub name: String,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChannelError {
    #[error("name cannot be empty")]
    EmptyName,
    #[error("name is too long")]
    NameTooLong,
    #[error("id must be greater than zero")]
    InvalidId,
}

pub fn validate_name(name: String) -> Result<String, ChannelError> {
    if name.trim().is_empty() {
        return Err(ChannelError::EmptyName);
    }
    if name.len() > MAX_NAME_BYTES {
        return Err(ChannelError::NameTooLong);
    }
    Ok(name)
}
