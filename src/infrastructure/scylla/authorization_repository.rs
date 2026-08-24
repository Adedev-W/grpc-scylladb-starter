use crate::application::authorization::{
    Action, AuthorizationError, Authorizer, Principal, Resource, role_allows,
};
use async_trait::async_trait;
use scylla::client::session::Session;
use std::sync::Arc;

pub struct ScyllaAuthorizer {
    session: Arc<Session>,
}

impl ScyllaAuthorizer {
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Authorizer for ScyllaAuthorizer {
    async fn authorize(
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

        let result = self
            .session
            .query_unpaged(
                "SELECT role FROM auth_subject_roles WHERE subject = ?",
                (principal.subject.as_str(),),
            )
            .await
            .map_err(|error| AuthorizationError::Storage(error.to_string()))?
            .into_rows_result()
            .map_err(|error| AuthorizationError::Storage(error.to_string()))?;
        let mut rows = result
            .rows::<(String,)>()
            .map_err(|error| AuthorizationError::Storage(error.to_string()))?;
        let role = rows
            .next()
            .transpose()
            .map_err(|error| AuthorizationError::Storage(error.to_string()))?
            .map(|row| row.0);

        if role
            .as_deref()
            .is_some_and(|role| role_allows(role, action))
        {
            Ok(())
        } else {
            Err(AuthorizationError::AccessDenied)
        }
    }
}
