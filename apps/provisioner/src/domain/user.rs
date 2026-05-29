use crate::domain::Username;

#[derive(Clone, PartialEq, Eq)]
pub struct ProvisionedUser {
    username: Username,
    password: String,
}

impl ProvisionedUser {
    pub fn new(username: Username, password: impl Into<String>) -> Self {
        Self {
            username,
            password: password.into(),
        }
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

impl std::fmt::Debug for ProvisionedUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvisionedUser")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}
