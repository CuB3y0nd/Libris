use std::fmt::{self, Display, Formatter};

use crate::error::{HarnessError, Result};

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Username(String);

impl Username {
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let raw = value.as_ref().trim();

        if raw.is_empty() {
            return Err(HarnessError::new("username cannot be empty"));
        }

        if raw.len() > 64 {
            return Err(HarnessError::new("username cannot exceed 64 bytes"));
        }

        if raw == "." || raw == ".." || raw.contains("..") {
            return Err(HarnessError::new("username cannot contain path traversal"));
        }

        if raw.starts_with('.') || raw.ends_with('.') {
            return Err(HarnessError::new("username cannot start or end with dot"));
        }

        for byte in raw.bytes() {
            let allowed = byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-');
            if !allowed {
                return Err(HarnessError::new(format!(
                    "username contains invalid character: {}",
                    byte as char
                )));
            }
        }

        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Username {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Username").field(&self.as_str()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_safe_usernames() -> Result<()> {
        for value in ["alice", "bob-01", "carol.dev", "dave_ops"] {
            assert_eq!(Username::parse(value)?.as_str(), value);
        }
        Ok(())
    }

    #[test]
    fn rejects_unsafe_usernames() {
        for value in [
            "",
            ".",
            "..",
            "../alice",
            "alice/bob",
            "alice\\bob",
            "alice bob",
            "a:b",
            ".alice",
            "alice.",
            "a..b",
        ] {
            assert!(
                Username::parse(value).is_err(),
                "value should fail: {value}"
            );
        }
    }
}
