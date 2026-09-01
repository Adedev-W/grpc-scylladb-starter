use thiserror::Error;
use uuid::Uuid;

pub const MAX_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChannelError {
    #[error("name cannot be empty")]
    EmptyName,
    #[error("name is too long")]
    NameTooLong,
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

#[cfg(test)]
mod tests {
    use super::{ChannelError, MAX_NAME_BYTES, validate_name};

    #[test]
    fn names_must_contain_non_whitespace() {
        assert_eq!(validate_name("  ".into()), Err(ChannelError::EmptyName));
    }

    #[test]
    fn names_are_limited_by_bytes() {
        let name = "a".repeat(MAX_NAME_BYTES + 1);
        assert_eq!(validate_name(name), Err(ChannelError::NameTooLong));
    }

    #[test]
    fn valid_names_are_preserved() {
        let name = " engineering ".to_string();
        assert_eq!(validate_name(name.clone()), Ok(name));
    }
}
