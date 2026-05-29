use crate::error::{HarnessError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpEndpoint {
    pub host: String,
    pub port: u16,
    pub base_path: String,
}

impl HttpEndpoint {
    pub fn parse(value: &str) -> Result<Self> {
        let value = value.trim();
        let without_scheme = value.strip_prefix("http://").ok_or_else(|| {
            HarnessError::new("only http:// endpoints are supported for the internal SFTPGo API")
        })?;

        let (authority, path) = match without_scheme.split_once('/') {
            Some((authority, path)) => (authority, format!("/{path}")),
            None => (without_scheme, String::new()),
        };

        if authority.is_empty() {
            return Err(HarnessError::new("http endpoint host cannot be empty"));
        }

        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) if !host.is_empty() => (host.to_string(), port.parse::<u16>()?),
            _ => (authority.to_string(), 80),
        };

        let base_path = normalize_base_path(&path);

        Ok(Self {
            host,
            port,
            base_path,
        })
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn path(&self, suffix: &str) -> String {
        let suffix = if suffix.starts_with('/') {
            suffix.to_string()
        } else {
            format!("/{suffix}")
        };
        if self.base_path.is_empty() || self.base_path == "/" {
            suffix
        } else {
            format!("{}{}", self.base_path, suffix)
        }
    }
}

pub fn percent_encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let allowed = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~');
        if allowed {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex(byte >> 4));
            encoded.push(hex(byte & 0x0f));
        }
    }
    encoded
}

fn normalize_base_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed == "/" {
        String::new()
    } else {
        trimmed.to_string()
    }
}

fn hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + (value - 10)) as char,
        _ => '0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_endpoint_with_port() -> Result<()> {
        let endpoint = HttpEndpoint::parse("http://sftpgo:8080/base/")?;
        assert_eq!(endpoint.host, "sftpgo");
        assert_eq!(endpoint.port, 8080);
        assert_eq!(endpoint.path("/api/v2/token"), "/base/api/v2/token");
        Ok(())
    }

    #[test]
    fn rejects_https_for_internal_client() {
        assert!(HttpEndpoint::parse("https://sftpgo.example.com").is_err());
    }

    #[test]
    fn encodes_path_segment() {
        assert_eq!(percent_encode_path_segment("a b/c"), "a%20b%2Fc");
    }
}
