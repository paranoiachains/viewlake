#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use crate::comms::client::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use windows::core::Result;

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
        let request_handle = WinHttpRequest::new(&connection, method, path)?;

        request_handle.send(headers, body)?;
        request_handle.receive()?;

        let mut body = Option::Some(Vec::new());
        if let Ok((buffer, bytes_read)) = request_handle.read() {
            body.as_mut()
                .unwrap()
                .extend_from_slice(&buffer[..bytes_read as usize]);
        } else {
            body = None;
        }

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
