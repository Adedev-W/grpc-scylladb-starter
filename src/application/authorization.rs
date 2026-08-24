use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub resource_type: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    List,
    Create,
    Update,
    Delete,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthorizationError {
    #[error("principal subject cannot be empty")]
    EmptySubject,
    #[error("access denied")]
    AccessDenied,
    #[error("authorization storage failed: {0}")]
    Storage(String),
}

#[async_trait]
pub trait Authorizer: Send + Sync {
    async fn authorize(
        &self,
        principal: &Principal,
        resource: &Resource,
        action: Action,
    ) -> Result<(), AuthorizationError>;
}

pub fn role_allows(role: &str, action: Action) -> bool {
    match role {
        "reader" => matches!(action, Action::Read | Action::List),
        "writer" => matches!(
            action,
            Action::Read | Action::List | Action::Create | Action::Update
        ),
        "admin" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, role_allows};

    #[test]
    fn roles_have_clear_permissions() {
        println!("[RBAC] reader: read,list | writer: read,list,create,update | admin: all");
        assert!(role_allows("reader", Action::Read));
        assert!(!role_allows("reader", Action::Delete));
        assert!(role_allows("writer", Action::Update));
        assert!(!role_allows("writer", Action::Delete));
        assert!(role_allows("admin", Action::Delete));
        assert!(!role_allows("unknown", Action::Read));
    }
}
