#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use crate::comms::client::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use std::os::raw::c_void;
use windows::core::Result;

use std::collections::HashMap;

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    session: WinHttpSession,
    connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: &'static str = "SomeAgent"; // TODO: randomize user-agent

/// Abstraction over WinHttpRequest.
pub struct RequestHandle(pub WinHttpRequest);

impl RequestHandle {
    fn read(&self) -> Result<Option<Vec<u8>>> {
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

    fn send(&self, headers: Option<HashMap<String, String>>, body: Option<&str>) -> Result<()> {
        self.0.send(headers, body)
    }

    fn status_code(&self) -> Result<u32> {
        self.0.status_code()
    }

    fn headers(&self) -> Result<HashMap<String, String>> {
        self.0.headers()
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

    /// Sends request and returns `Response` struct.
    pub fn send_and_read(
        &mut self,
        hostname: &str,
        port: u16,
        method: &str,
        path: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&str>,
    ) -> Result<Response> {
        if self
            .connection
            .as_ref()
            .map_or(true, |c| c.hostname != hostname)
        {
            self.connection = Some(WinHttpConnection::new(&self.session, &hostname, port)?);
        }
        let connection = self.connection.as_ref().unwrap();
        let request_handle = RequestHandle(WinHttpRequest::new(&connection, method, path)?);

        request_handle.send(headers, body)?;
        request_handle.receive()?;

        let body = request_handle.read().unwrap();
        let status_code = request_handle.status_code()?;
        let headers = request_handle.headers()?;

        Ok(Response::new(status_code, headers, body))
    }
}

pub struct Request<'a> {
    pub hostname: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<&'a str>,
}

impl<'a> Request<'a> {
    pub fn new(
        hostname: &'a str,
        port: u16,
        method: &'a str,
        path: &'a str,
        headers: Option<HashMap<String, String>>,
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
    pub code: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Response {
    fn new(code: u32, headers: HashMap<String, String>, body: Option<Vec<u8>>) -> Self {
        let body = body.map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        Response {
            code,
            headers,
            body,
        }
    }
}
