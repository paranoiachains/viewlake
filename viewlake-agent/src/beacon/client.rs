#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use windows::core::Result;
use winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};

use std::collections::HashMap;

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    session: WinHttpSession,
    connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: &'static str = "SomeAgent"; // TODO: randomize user-agent

impl Client {
    /// Initializes WinHttpSession
    pub fn new() -> Result<Self> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
        })
    }

    /// Sends request and returns `Response` struct.
    pub fn request(&mut self, request: &Request) -> Result<Response> {
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
        let request_handle = WinHttpRequest::new(&connection, request.method, request.path)?;

        request_handle.send(request.headers, request.body)?;
        request_handle.receive()?;
        let body = match request_handle.read() {
            Ok(bytes) => Some(bytes),
            Err(_) => None,
        };

        let status_code = request_handle.status_code()?;
        let headers = request_handle.headers()?;

        Ok(Response::new(status_code, headers, body))
    }

    /// Return home's hostname if present
    pub fn home(&self) -> Option<&str> {
        if let Some(ref conn) = self.connection {
            return Some(&conn.hostname);
        }
        None
    }
}

pub struct Request<'a> {
    pub hostname: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,
    pub headers: Option<&'a HashMap<String, String>>,
    pub body: Option<&'a str>,
}

impl<'a> Request<'a> {
    pub fn new(
        hostname: &'a str,
        port: u16,
        method: &'a str,
        path: &'a str,
        headers: Option<&'a HashMap<String, String>>,
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
