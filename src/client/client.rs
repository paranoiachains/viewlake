use super::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use std::{collections::HashMap, io::Write};
use windows::core::{Error, HSTRING};

pub struct Client {
    pub session: WinHttpSession,
    pub connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: &'static str = "SomeAgent"; // TODO: randomize user-agent

pub struct RequestHandle(WinHttpRequest);

impl Client {
    pub fn new() -> Result<Self, Error> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
        })
    }

    pub fn send_request(&mut self, request: Request) -> Result<RequestHandle, Error> {
        if self
            .connection
            .as_ref()
            .map_or(true, |c| c.hostname != request.hostname)
        {
            self.connection = Some(WinHttpConnection::new(
                &self.session,
                &request.hostname,
                request.port,
            )?);
        }
        let connection = self.connection.as_ref().unwrap();
        let win_request = WinHttpRequest::new(&connection, request.method, request.path)?;

        win_request.send(request.headers, request.body)?;
        Ok(RequestHandle(win_request))
    }

    pub fn receive_response(&self, handle: &RequestHandle) -> Result<Response, Error> {
        handle.0.receive()?;

        let mut buf = [0u8; 4096];
        handle
            .0
            .read(buf.as_mut_ptr() as *mut _, buf.len() as u32)?;

        match std::str::from_utf8(&buf) {
            Ok(data) => Ok(Response::new(data.to_string())),
            Err(_) => Err(Error::new(windows::core::HRESULT(1), HSTRING::new())),
        }
    }
}

pub struct Request<'a> {
    pub hostname: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,
    pub headers: Option<Vec<&'a str>>,
    pub body: Option<&'a str>,
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
    pub fn new(raw: String) -> Self {
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

    pub fn into_stdout(&self) {
        std::io::stdout().write_all(self.data.as_bytes()).unwrap();
    }
}

// Warning: These tests send actual requests (integrational)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client() {
        Client::new().expect("Failed to create client");
    }

    #[test]
    fn send_request_with_body_and_headers() {
        let mut client = Client::new().expect("Failed to create client");

        let request = Request::new(
            "httpbin.org",
            443,
            "GET",
            "/anything",
            Some(vec!["Header1: 1", "Header2: 2"]),
            Some("Body"),
        );

        let handle = client
            .send_request(request)
            .expect("Failed to send request");
        let response = client
            .receive_response(&handle)
            .expect("Failed to receive response");

        assert!(!response.data.is_empty())
    }

    #[test]
    fn send_request_without_body_and_headers() {
        let mut client = Client::new().expect("Failed to create client");

        let request = Request::new("httpbin.org", 443, "GET", "/anything", None, None);

        let handle = client
            .send_request(request)
            .expect("Failed to send request");
        let response = client
            .receive_response(&handle)
            .expect("Failed to receive response");

        assert!(!response.data.is_empty());
    }

    #[test]
    fn reuse_connection_for_same_host() {
        let mut client = Client::new().expect("Failed to create client");

        let request_1 = Request::new("httpbin.org", 443, "GET", "/anything", None, None);
        let handle_1 = client
            .send_request(request_1)
            .expect("Failed to send request");
        let response_1 = client
            .receive_response(&handle_1)
            .expect("Failed to receive response");

        assert!(!response_1.data.is_empty());

        let request_2 = Request::new("httpbin.org", 443, "POST", "/anything", None, None);
        let handle_2 = client
            .send_request(request_2)
            .expect("Failed to send request");
        let response_2 = client
            .receive_response(&handle_2)
            .expect("Failed to receive response");

        assert!(!response_2.data.is_empty())
    }

    #[test]
    fn new_connection_for_different_host() {
        let mut client = Client::new().expect("Failed to create client");

        let request_1 = Request::new("httpbin.org", 443, "GET", "/anything", None, None);
        let handle_1 = client
            .send_request(request_1)
            .expect("Failed to send request");
        let response_1 = client
            .receive_response(&handle_1)
            .expect("Failed to receive response");

        assert!(!response_1.data.is_empty());

        let request_2 = Request::new("example.com", 443, "GET", "/", None, None);
        let _handle_2 = client
            .send_request(request_2)
            .expect("Failed to send request");
        let response_2 = client
            .receive_response(&_handle_2)
            .expect("Failed to receive response");

        assert!(!response_2.data.is_empty())
    }

    #[test]
    fn test_request_build() {
        let raw = String::from(
            "HTTP/1.1 200 OK\r\n
            Date: Tue, 02 Sep 2025 19:20:00 GMT\r\n
            Content-Type: text/plain; charset=utf-8\r\n
            Content-Length: 13\r\n
            Connection: close\r\n
            \r\n
            Hello, world!",
        );

        let response = Response::new(raw);
        assert_eq!(response.code, 200);
        assert_eq!(response.body, "Hello, world!");

        let mut expected_headers = std::collections::HashMap::new();
        expected_headers.insert(
            "Date".to_string(),
            "Tue, 02 Sep 2025 19:20:00 GMT".to_string(),
        );
        expected_headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );
        expected_headers.insert("Content-Length".to_string(), "13".to_string());
        expected_headers.insert("Connection".to_string(), "close".to_string());

        assert_eq!(response.headers, expected_headers);
    }
}
