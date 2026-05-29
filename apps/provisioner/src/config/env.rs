use std::env;
use std::path::PathBuf;

use crate::domain::S3Settings;
use crate::error::{HarnessError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SftpGoConfig {
    pub endpoint: String,
    pub admin_user: String,
    pub admin_password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub sftpgo: SftpGoConfig,
    pub s3: S3Settings,
    pub user_file: PathBuf,
    pub dry_run: bool,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Self::from_getter(|key| env::var(key).ok())
    }

    pub fn from_getter<F>(mut get: F) -> Result<Self>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let sftpgo = SftpGoConfig {
            endpoint: required(&mut get, "SFTPGO_ENDPOINT")?,
            admin_user: required(&mut get, "SFTPGO_ADMIN_USER")?,
            admin_password: required(&mut get, "SFTPGO_ADMIN_PASSWORD")?,
        };

        let endpoint = optional(&mut get, "S3_ENDPOINT");
        let force_path_style = optional(&mut get, "S3_FORCE_PATH_STYLE")
            .map(|value| parse_bool("S3_FORCE_PATH_STYLE", &value))
            .transpose()?
            .unwrap_or_else(|| endpoint.as_deref().is_some_and(|value| !value.is_empty()));

        let s3 = S3Settings {
            bucket: required(&mut get, "S3_BUCKET")?,
            region: required(&mut get, "S3_REGION")?,
            endpoint,
            access_key: required(&mut get, "S3_ACCESS_KEY")?,
            access_secret: required(&mut get, "S3_SECRET_KEY")?,
            prefix_base: optional(&mut get, "S3_PREFIX_BASE")
                .unwrap_or_else(|| "users".to_string()),
            force_path_style,
        };

        let user_file = PathBuf::from(
            optional(&mut get, "USER_FILE").unwrap_or_else(|| "/config/users.csv".to_string()),
        );

        let dry_run = optional(&mut get, "PROVISIONER_DRY_RUN")
            .map(|value| parse_bool("PROVISIONER_DRY_RUN", &value))
            .transpose()?
            .unwrap_or(false);

        let retry_count = optional(&mut get, "PROVISIONER_RETRY_COUNT")
            .unwrap_or_else(|| "60".to_string())
            .parse::<u32>()?;

        let retry_delay_ms = optional(&mut get, "PROVISIONER_RETRY_DELAY_MS")
            .unwrap_or_else(|| "1000".to_string())
            .parse::<u64>()?;

        Ok(Self {
            sftpgo,
            s3,
            user_file,
            dry_run,
            retry_count,
            retry_delay_ms,
        })
    }
}

fn required<F>(get: &mut F, key: &str) -> Result<String>
where
    F: FnMut(&str) -> Option<String>,
{
    optional(get, key).ok_or_else(|| HarnessError::new(format!("missing required env var {key}")))
}

fn optional<F>(get: &mut F, key: &str) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    get(key).map(|value| value.trim().to_string())
}

fn parse_bool(key: &str, value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(HarnessError::new(format!(
            "invalid boolean value for {key}: {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn reads_required_config_and_defaults() -> Result<()> {
        let pairs = HashMap::from([
            ("SFTPGO_ENDPOINT", "http://sftpgo:5757"),
            ("SFTPGO_ADMIN_USER", "admin"),
            ("SFTPGO_ADMIN_PASSWORD", "secret"),
            ("S3_BUCKET", "bucket"),
            ("S3_REGION", "us-east-1"),
            ("S3_ACCESS_KEY", "access"),
            ("S3_SECRET_KEY", "secret"),
        ]);

        let config =
            AppConfig::from_getter(|key| pairs.get(key).map(|value| (*value).to_string()))?;

        assert_eq!(config.s3.prefix_base, "users");
        assert!(!config.dry_run);
        assert_eq!(config.retry_count, 60);
        Ok(())
    }

    #[test]
    fn rejects_invalid_bool() {
        let pairs = HashMap::from([
            ("SFTPGO_ENDPOINT", "http://sftpgo:5757"),
            ("SFTPGO_ADMIN_USER", "admin"),
            ("SFTPGO_ADMIN_PASSWORD", "secret"),
            ("S3_BUCKET", "bucket"),
            ("S3_REGION", "us-east-1"),
            ("S3_ACCESS_KEY", "access"),
            ("S3_SECRET_KEY", "secret"),
            ("PROVISIONER_DRY_RUN", "maybe"),
        ]);

        let error = AppConfig::from_getter(|key| pairs.get(key).map(|value| (*value).to_string()))
            .err()
            .map(|value| value.to_string())
            .unwrap_or_default();

        assert!(error.contains("invalid boolean"));
    }
}
