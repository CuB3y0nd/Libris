use serde_json::{json, Map, Value};

use crate::domain::{ProvisionedUser, S3Settings};

pub fn build_user_payload(user: &ProvisionedUser, s3: &S3Settings, key_prefix: &str) -> Value {
    let mut s3_config = Map::new();
    s3_config.insert("bucket".to_string(), json!(s3.bucket.as_str()));
    s3_config.insert("region".to_string(), json!(s3.region.as_str()));
    s3_config.insert("access_key".to_string(), json!(s3.access_key.as_str()));
    s3_config.insert(
        "access_secret".to_string(),
        json!({
            "status": "Plain",
            "payload": s3.access_secret.as_str(),
        }),
    );
    s3_config.insert("key_prefix".to_string(), json!(key_prefix));
    s3_config.insert("force_path_style".to_string(), json!(s3.force_path_style));

    if let Some(endpoint) = &s3.endpoint {
        if !endpoint.is_empty() {
            s3_config.insert("endpoint".to_string(), json!(endpoint.as_str()));
        }
    }

    json!({
        "status": 1,
        "username": user.username().as_str(),
        "password": user.password(),
        "permissions": {
            "/": ["*"]
        },
        "filesystem": {
            "provider": 1,
            "s3config": Value::Object(s3_config)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ProvisionedUser, Username};
    use crate::error::Result;

    #[test]
    fn builds_sftpgo_s3_user_payload() -> Result<()> {
        let user = ProvisionedUser::new(Username::parse("alice")?, "alice-password");
        let s3 = S3Settings {
            bucket: "bucket".into(),
            region: "us-east-1".into(),
            endpoint: None,
            access_key: "access".into(),
            access_secret: "secret".into(),
            prefix_base: "users".into(),
            force_path_style: false,
        };

        let payload = build_user_payload(&user, &s3, "users/alice/");

        assert_eq!(payload["status"], 1);
        assert_eq!(payload["username"], "alice");
        assert_eq!(payload["filesystem"]["provider"], 1);
        assert_eq!(
            payload["filesystem"]["s3config"]["key_prefix"],
            "users/alice/"
        );
        assert_eq!(payload["permissions"]["/"][0], "*");
        Ok(())
    }
}
