use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::error::{HarnessError, Result};
use crate::infra::http::HttpEndpoint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn json_body(mut self, body: &serde_json::Value) -> Result<Self> {
        self.body = serde_json::to_vec(body)?;
        self.headers
            .push(("Content-Type".to_string(), "application/json".to_string()));
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        (200..=299).contains(&self.status)
    }

    pub fn body_as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.body)
            .map_err(|error| HarnessError::with_context("response body is not utf-8", error))
    }

    pub fn header_value(&self, needle: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(needle))
            .map(|(_, value)| value.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct HttpClient {
    endpoint: HttpEndpoint,
    timeout: Duration,
}

impl HttpClient {
    pub fn new(endpoint: HttpEndpoint) -> Self {
        Self {
            endpoint,
            timeout: Duration::from_secs(10),
        }
    }

    pub fn send(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let mut stream = TcpStream::connect(self.endpoint.socket_addr())?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        let wire = self.render_request(request);
        stream.write_all(&wire)?;
        stream.flush()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        parse_response(&response)
    }

    fn render_request(&self, request: &HttpRequest) -> Vec<u8> {
        let mut wire = Vec::new();
        let path = self.endpoint.path(&request.path);

        write_ascii(
            &mut wire,
            &format!("{} {} HTTP/1.1\r\n", request.method, path),
        );
        write_ascii(&mut wire, &format!("Host: {}\r\n", self.endpoint.host));
        write_ascii(&mut wire, "Connection: close\r\n");
        write_ascii(&mut wire, "Accept: application/json\r\n");
        write_ascii(
            &mut wire,
            &format!("Content-Length: {}\r\n", request.body.len()),
        );

        for (name, value) in &request.headers {
            write_ascii(&mut wire, &format!("{name}: {value}\r\n"));
        }

        write_ascii(&mut wire, "\r\n");
        wire.extend_from_slice(&request.body);
        wire
    }
}

fn write_ascii(buffer: &mut Vec<u8>, value: &str) {
    buffer.extend_from_slice(value.as_bytes());
}

fn parse_response(bytes: &[u8]) -> Result<HttpResponse> {
    let split = find_header_body_split(bytes)
        .ok_or_else(|| HarnessError::new("invalid http response: missing header terminator"))?;
    let header_bytes = &bytes[..split];
    let body_bytes = &bytes[split + 4..];
    let header_text = std::str::from_utf8(header_bytes)
        .map_err(|error| HarnessError::with_context("invalid response headers", error))?;

    let mut lines = header_text.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| HarnessError::new("invalid http response: missing status line"))?;
    let status = parse_status(status_line)?;

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| HarnessError::new("invalid http response header"))?;
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }

    let mut response = HttpResponse {
        status,
        headers,
        body: body_bytes.to_vec(),
    };

    if response
        .header_value("Transfer-Encoding")
        .is_some_and(|value| value.eq_ignore_ascii_case("chunked"))
    {
        response.body = decode_chunked(&response.body)?;
    }

    Ok(response)
}

fn parse_status(status_line: &str) -> Result<u16> {
    let mut parts = status_line.split_whitespace();
    let version = parts
        .next()
        .ok_or_else(|| HarnessError::new("invalid http response: missing version"))?;
    if !version.starts_with("HTTP/") {
        return Err(HarnessError::new("invalid http response version"));
    }
    let status = parts
        .next()
        .ok_or_else(|| HarnessError::new("invalid http response: missing status"))?
        .parse::<u16>()?;
    Ok(status)
}

fn find_header_body_split(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn decode_chunked(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = 0;
    let mut output = Vec::new();

    loop {
        let line_end = find_crlf(input, cursor)
            .ok_or_else(|| HarnessError::new("invalid chunked response: missing chunk size"))?;
        let size_text = std::str::from_utf8(&input[cursor..line_end])
            .map_err(|error| HarnessError::with_context("invalid chunk size", error))?;
        let size = usize::from_str_radix(size_text.trim(), 16)
            .map_err(|error| HarnessError::with_context("invalid chunk size", error))?;
        cursor = line_end + 2;

        if size == 0 {
            break;
        }

        let chunk_end = cursor + size;
        if chunk_end + 2 > input.len() {
            return Err(HarnessError::new(
                "invalid chunked response: chunk exceeds body",
            ));
        }
        output.extend_from_slice(&input[cursor..chunk_end]);
        cursor = chunk_end + 2;
    }

    Ok(output)
}

fn find_crlf(input: &[u8], start: usize) -> Option<usize> {
    input[start..]
        .windows(2)
        .position(|window| window == b"\r\n")
        .map(|offset| start + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_content_length_response() -> Result<()> {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
        let response = parse_response(raw)?;
        assert_eq!(response.status, 200);
        assert_eq!(response.body_as_str()?, "ok");
        Ok(())
    }

    #[test]
    fn parses_chunked_response() -> Result<()> {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        let response = parse_response(raw)?;
        assert_eq!(response.body_as_str()?, "Wikipedia");
        Ok(())
    }
}
