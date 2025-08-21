use serde::{Deserialize, Serialize};
use std::fmt;

use crate::domain::value_object::user_id::UserId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserContext {
    user_id: UserId,
    username: Option<String>,
    role: Option<String>,
}

impl UserContext {
    pub fn new(user_id: UserId, username: Option<String>, role: Option<String>) -> Self {
        Self {
            user_id,
            username,
            role,
        }
    }

    pub fn anonymous() -> Option<Self> {
        None
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    pub fn is_admin(&self) -> bool {
        self.role
            .as_ref()
            .map(|r| r.to_lowercase() == "admin")
            .unwrap_or(false)
    }
}

impl fmt::Display for UserContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.username, &self.role) {
            (Some(username), Some(role)) => write!(f, "{}({}) [{}]", username, self.user_id, role),
            (Some(username), None) => write!(f, "{}({})", username, self.user_id),
            (None, Some(role)) => write!(f, "{}[{}]", self.user_id, role),
            (None, None) => write!(f, "{}", self.user_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_context_new() {
        let user_id = UserId::new("user_1");
        let context = UserContext::new(
            user_id.clone(),
            Some("john_doe".to_string()),
            Some("admin".to_string()),
        );

        assert_eq!(context.user_id(), &user_id);
        assert_eq!(context.username(), Some("john_doe"));
        assert_eq!(context.role(), Some("admin"));
        assert!(context.is_admin());
    }

    #[test]
    fn test_user_context_minimal() {
        let user_id = UserId::new("user_1");
        let context = UserContext::new(user_id.clone(), None, None);

        assert_eq!(context.user_id(), &user_id);
        assert_eq!(context.username(), None);
        assert_eq!(context.role(), None);
        assert!(!context.is_admin());
    }

    #[test]
    fn test_is_admin() {
        let user_id = UserId::new("user_1");
        
        let admin_context = UserContext::new(
            user_id.clone(),
            None,
            Some("admin".to_string()),
        );
        assert!(admin_context.is_admin());

        let admin_context_caps = UserContext::new(
            user_id.clone(),
            None,
            Some("ADMIN".to_string()),
        );
        assert!(admin_context_caps.is_admin());

        let user_context = UserContext::new(
            user_id.clone(),
            None,
            Some("user".to_string()),
        );
        assert!(!user_context.is_admin());

        let no_role_context = UserContext::new(user_id, None, None);
        assert!(!no_role_context.is_admin());
    }
}