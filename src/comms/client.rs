#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use crate::comms::client::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use std::os::raw::c_void;
use windows::core::Result;

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    session: WinHttpSession,
    connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: &'static str = "SomeAgent"; // TODO: randomize user-agent

pub struct RequestHandle(pub WinHttpRequest);

impl RequestHandle {
    pub fn read(&self) -> Result<Option<Vec<u8>>> {
        let mut body = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            let bytes_read = self
                .0
                .read(buf.as_mut_ptr() as *mut c_void, buf.len() as u32)?;

            if bytes_read == 0 {
                break; // no more data
            }

            body.extend_from_slice(&buf[..bytes_read as usize]);
        }

        if body.is_empty() {
            Ok(None)
        } else {
            Ok(Some(body))
        }
    }

    fn receive(&self) -> Result<()> {
        self.0.receive()
    }
}

impl Client {
    pub fn new() -> Result<Self> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
        })
    }

    /// Sends request and returns RequestHandle, which is supposed to be passed to receive_response
    pub fn send(
        &mut self,
        hostname: &str,
        port: u16,
        method: &str,
        path: &str,
        headers: Option<Vec<&str>>,
        body: Option<&str>,
    ) -> Result<RequestHandle> {
        if self
            .connection
            .as_ref()
            .map_or(true, |c| c.hostname != hostname)
        {
            self.connection = Some(WinHttpConnection::new(&self.session, &hostname, port)?);
        }
        let connection = self.connection.as_ref().unwrap();
        let win_request = WinHttpRequest::new(&connection, method, path)?;

        win_request.send(headers, body)?;
        Ok(RequestHandle(win_request))
    }

    /// Receives response using provided RequestHandle obtained from send_request func
    pub fn receive(&self, handle: &RequestHandle) -> Result<()> {
        handle.receive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HOST: &str = "www.example.com";
    const TEST_PORT: u16 = 443;
    const TEST_PATH: &str = "/";

    #[test]
    fn client_creation() {
        assert!(Client::new().is_ok())
    }

    #[test]
    fn send_get_request() {
        let mut client = Client::new().unwrap();
        let request = client
            .send(TEST_HOST, TEST_PORT, "GET", TEST_PATH, None, None)
            .expect("Failed to send GET request");

        let response_text = client
            .receive(&request)
            .expect("Failed to receive response");

        assert!(!response_text.is_some(), "Response body is empty");
        assert!(
            response_text.unwrap().contains("Example Domain"),
            "Unexpected response content"
        );
    }

    #[test]
    fn send_request_with_headers() {
        let mut client = Client::new().unwrap();
        let headers = vec!["User-Agent: RustTestClient"];
        let request = client
            .send(TEST_HOST, TEST_PORT, "GET", TEST_PATH, Some(headers), None)
            .expect("Failed to send GET request with headers");

        let response_text = client
            .receive(&request)
            .expect("Failed to receive response");

        assert!(!response_text.is_some(), "Response body is empty");
    }

    #[test]
    fn send_post_request_with_body() {
        let mut client = Client::new().unwrap();
        let body = "Test body content";

        let request = client
            .send(TEST_HOST, TEST_PORT, "POST", TEST_PATH, None, Some(body))
            .expect("Failed to send POST request");

        let response_text = client
            .receive(&request)
            .expect("Failed to receive response");

        assert!(!response_text.is_some(), "Response body is empty");
    }
}
