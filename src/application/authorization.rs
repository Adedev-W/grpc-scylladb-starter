use std::collections::HashMap;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Reader,
    Writer,
    Admin,
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryAuthorizer {
    subjects: HashMap<String, Vec<Role>>,
}

impl InMemoryAuthorizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_subject(mut self, subject: impl Into<String>, roles: Vec<Role>) -> Self {
        self.subjects.insert(subject.into(), roles);
        self
    }

    pub fn from_bindings(bindings: &str) -> Self {
        bindings
            .split(';')
            .filter_map(|binding| binding.split_once('='))
            .fold(Self::new(), |authorizer, (subject, roles)| {
                let roles = roles
                    .split('|')
                    .filter_map(|role| match role.trim() {
                        "reader" => Some(Role::Reader),
                        "writer" => Some(Role::Writer),
                        "admin" => Some(Role::Admin),
                        _ => None,
                    })
                    .collect();
                authorizer.bind_subject(subject.trim(), roles)
            })
    }

    pub fn authorize(
        &self,
        principal: &Principal,
        resource: &Resource,
        action: Action,
    ) -> Result<(), AuthorizationError> {
        if principal.subject.trim().is_empty() {
            return Err(AuthorizationError::EmptySubject);
        }

        if resource.resource_type != "channel" {
            return Err(AuthorizationError::AccessDenied);
        }

        let roles = self
            .subjects
            .get(&principal.subject)
            .ok_or(AuthorizationError::AccessDenied)?;

        if roles.iter().any(|role| role.allows(action)) {
            Ok(())
        } else {
            Err(AuthorizationError::AccessDenied)
        }
    }
}

impl Role {
    fn allows(self, action: Action) -> bool {
        match self {
            Self::Reader => matches!(action, Action::Read | Action::List),
            Self::Writer => matches!(
                action,
                Action::Read | Action::List | Action::Create | Action::Update
            ),
            Self::Admin => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, AuthorizationError, InMemoryAuthorizer, Principal, Resource, Role};

    fn channel(id: Option<&str>) -> Resource {
        Resource {
            resource_type: "channel".to_string(),
            id: id.map(str::to_string),
        }
    }

    #[test]
    fn reader_can_read_and_list_channels() {
        let authorizer =
            InMemoryAuthorizer::new().bind_subject("reader.example", vec![Role::Reader]);
        let principal = Principal {
            subject: "reader.example".to_string(),
        };

        assert!(
            authorizer
                .authorize(&principal, &channel(Some("42")), Action::Read)
                .is_ok()
        );
        assert!(
            authorizer
                .authorize(&principal, &channel(None), Action::List)
                .is_ok()
        );
    }

    #[test]
    fn reader_cannot_delete() {
        let authorizer =
            InMemoryAuthorizer::new().bind_subject("reader.example", vec![Role::Reader]);
        let principal = Principal {
            subject: "reader.example".to_string(),
        };

        assert_eq!(
            authorizer.authorize(&principal, &channel(Some("42")), Action::Delete),
            Err(AuthorizationError::AccessDenied)
        );
    }

    #[test]
    fn unknown_subject_is_denied_by_default() {
        let principal = Principal {
            subject: "unknown.example".to_string(),
        };

        assert_eq!(
            InMemoryAuthorizer::new().authorize(&principal, &channel(None), Action::List),
            Err(AuthorizationError::AccessDenied)
        );
    }

    #[test]
    fn admin_can_delete_a_channel() {
        let authorizer = InMemoryAuthorizer::new().bind_subject("admin.example", vec![Role::Admin]);
        let principal = Principal {
            subject: "admin.example".to_string(),
        };

        assert!(
            authorizer
                .authorize(&principal, &channel(Some("42")), Action::Delete)
                .is_ok()
        );
    }
}
