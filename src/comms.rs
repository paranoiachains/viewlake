mod client;
use crate::comms::client::*;
use std::collections::HashMap;
use windows::core::Result;

pub struct Communicator {
    client: Client,
}

impl Communicator {
    pub fn new() -> Result<Self> {
        let client = Client::new()?;
        Ok(Self { client })
    }

    /// Send request and receive response
    pub fn request(&mut self, request: Request) -> Result<Response> {
        let handle = self.client.send(
            request.hostname,
            request.port,
            request.method,
            request.path,
            request.headers,
            request.body,
        )?;

        let raw_response = self.client.receive(&handle)?;

        Ok(Response::new(raw_response))
    }
}

pub struct Request<'a> {
    hostname: &'a str,
    port: u16,
    method: &'a str,
    path: &'a str,
    headers: Option<Vec<&'a str>>,
    body: Option<&'a str>,
}

impl<'a> Request<'a> {
    pub fn new(
        hostname: &'a str,
        port: u16,
        method: &'a str,
        path: &'a str,
        headers: Option<Vec<&'a str>>,
        body: Option<&'a str>,
    ) -> Self {
        Request {
            hostname,
            port,
            method,
            path,
            headers,
            body,
        }
    }
}

pub struct Response {
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    fn new(raw: String) -> Self {
        let mut lines = raw.lines();

        let status_line = lines.next().unwrap_or("");
        let mut parts = status_line.split_whitespace();
        let code = parts
            .next()
            .and_then(|c| c.parse::<u16>().ok())
            .unwrap_or(0);

        let mut headers = HashMap::new();
        for line in &mut lines {
            if line.trim().is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        let body_str = lines.collect::<Vec<_>>().join("\n");
        let body = body_str.into_bytes();

        Self {
            code,
            headers,
            body,
        }
    }
}
