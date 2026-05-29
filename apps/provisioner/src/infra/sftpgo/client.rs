use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::domain::Username;
use crate::error::{HarnessError, Result};
use crate::infra::http::{
    basic_auth_value, percent_encode_path_segment, HttpClient, HttpEndpoint, HttpRequest,
};

#[derive(Debug, Clone)]
pub struct SftpGoClient {
    http: HttpClient,
    admin_user: String,
    admin_password: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

impl SftpGoClient {
    pub fn new(endpoint: String, admin_user: String, admin_password: String) -> Result<Self> {
        let endpoint = HttpEndpoint::parse(&endpoint)?;
        Ok(Self {
            http: HttpClient::new(endpoint),
            admin_user,
            admin_password,
        })
    }

    pub fn acquire_token_with_retry(
        &self,
        retry_count: u32,
        retry_delay_ms: u64,
    ) -> Result<String> {
        let mut last_error = HarnessError::new("token acquisition was not attempted");

        for _ in 0..retry_count.max(1) {
            match self.acquire_token_once() {
                Ok(token) => return Ok(token),
                Err(error) => {
                    last_error = error;
                    thread::sleep(Duration::from_millis(retry_delay_ms));
                }
            }
        }

        Err(HarnessError::with_context(
            "failed to acquire SFTPGo admin token",
            last_error,
        ))
    }

    pub fn upsert_user(&self, token: &str, username: &Username, payload: &Value) -> Result<()> {
        if self.user_exists(token, username)? {
            self.update_user(token, username, payload)
        } else {
            self.create_user(token, payload)
        }
    }

    fn acquire_token_once(&self) -> Result<String> {
        let request = HttpRequest::new("GET", "/api/v2/token").header(
            "Authorization",
            basic_auth_value(&self.admin_user, &self.admin_password),
        );
        let response = self.http.send(&request)?;
        if !response.is_success() {
            return Err(HarnessError::new(format!(
                "SFTPGo token endpoint returned HTTP {}",
                response.status
            )));
        }
        let token: TokenResponse = serde_json::from_slice(&response.body)?;
        if token.access_token.is_empty() {
            return Err(HarnessError::new(
                "SFTPGo token response had empty access_token",
            ));
        }
        Ok(token.access_token)
    }

    fn user_exists(&self, token: &str, username: &Username) -> Result<bool> {
        let path = format!(
            "/api/v2/users/{}",
            percent_encode_path_segment(username.as_str())
        );
        let request = HttpRequest::new("GET", path).header("Authorization", bearer(token));
        let response = self.http.send(&request)?;

        match response.status {
            200..=299 => Ok(true),
            404 => Ok(false),
            status => Err(HarnessError::new(format!(
                "SFTPGo user lookup for '{}' returned HTTP {}",
                username.as_str(),
                status
            ))),
        }
    }

    fn create_user(&self, token: &str, payload: &Value) -> Result<()> {
        let request = HttpRequest::new("POST", "/api/v2/users")
            .header("Authorization", bearer(token))
            .json_body(payload)?;
        let response = self.http.send(&request)?;
        if response.is_success() {
            Ok(())
        } else {
            Err(HarnessError::new(format!(
                "SFTPGo create user returned HTTP {}",
                response.status
            )))
        }
    }

    fn update_user(&self, token: &str, username: &Username, payload: &Value) -> Result<()> {
        let path = format!(
            "/api/v2/users/{}",
            percent_encode_path_segment(username.as_str())
        );
        let request = HttpRequest::new("PUT", path)
            .header("Authorization", bearer(token))
            .json_body(payload)?;
        let response = self.http.send(&request)?;
        if response.is_success() {
            Ok(())
        } else {
            Err(HarnessError::new(format!(
                "SFTPGo update user '{}' returned HTTP {}",
                username.as_str(),
                response.status
            )))
        }
    }
}

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}
