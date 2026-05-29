use crate::domain::Username;

#[derive(Clone, PartialEq, Eq)]
pub struct S3Settings {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub access_secret: String,
    pub prefix_base: String,
    pub force_path_style: bool,
}

impl S3Settings {
    pub fn prefix_for(&self, username: &Username) -> String {
        let base = self.prefix_base.trim_matches('/');
        if base.is_empty() {
            format!("{}/", username.as_str())
        } else {
            format!("{base}/{}/", username.as_str())
        }
    }
}

impl std::fmt::Debug for S3Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Settings")
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("endpoint", &self.endpoint)
            .field("access_key", &"<redacted>")
            .field("access_secret", &"<redacted>")
            .field("prefix_base", &self.prefix_base)
            .field("force_path_style", &self.force_path_style)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    fn derives_slash_normalized_prefix() -> Result<()> {
        let settings = S3Settings {
            bucket: "bucket".into(),
            region: "us-east-1".into(),
            endpoint: None,
            access_key: "access".into(),
            access_secret: "secret".into(),
            prefix_base: "/users/".into(),
            force_path_style: false,
        };
        let username = Username::parse("alice")?;
        assert_eq!(settings.prefix_for(&username), "users/alice/");
        Ok(())
    }
}
