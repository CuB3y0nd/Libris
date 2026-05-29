use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;

use zotero_s3_webdav_provisioner::error::{HarnessError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct MockHttpServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl MockHttpServer {
    pub fn start(responses: Vec<String>) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);

        thread::spawn(move || {
            for response in responses {
                let Ok((mut stream, _peer)) = listener.accept() else {
                    break;
                };
                let mut buffer = [0_u8; 8192];
                let Ok(read) = stream.read(&mut buffer) else {
                    break;
                };
                if let Some(recorded) = parse_request(&buffer[..read]) {
                    if let Ok(mut locked) = requests_for_thread.lock() {
                        locked.push(recorded);
                    }
                }
                let _ = stream.write_all(response.as_bytes());
            }
        });

        Ok(Self { addr, requests })
    }

    pub fn endpoint(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn requests(&self) -> Result<Vec<RecordedRequest>> {
        self.requests
            .lock()
            .map(|guard| guard.clone())
            .map_err(|_| HarnessError::new("mock request lock poisoned"))
    }
}

fn parse_request(bytes: &[u8]) -> Option<RecordedRequest> {
    let split = bytes.windows(4).position(|window| window == b"\r\n\r\n")?;
    let header = std::str::from_utf8(&bytes[..split]).ok()?;
    let mut lines = header.lines();
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let mut headers = Vec::new();
    for line in lines {
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    let body = bytes[split + 4..].to_vec();
    Some(RecordedRequest {
        method,
        path,
        headers,
        body,
    })
}

pub fn json_response(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}
