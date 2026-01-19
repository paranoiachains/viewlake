#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use windows::core::{Error, Result};
use winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    session: WinHttpSession,
    connection: Option<(Box<str>, u16, WinHttpConnection)>, // Store hostname with connection
    // (hostname, port, connection)
    request: Option<WinHttpRequest>,
}

const DEFAULT_AGENT: &'static str = "SomeAgent"; // TODO: randomize user-agent

impl Client {
    /// Initializes WinHttpSession
    pub fn new() -> Result<Self> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
            request: None,
        })
    }

    /// Sends request.
    pub fn request(&mut self, req: &Request) -> Result<()> {
        let reuse = self
            .connection
            .as_ref()
            .map(|(host, port, _)| host.as_ref() == req.hostname && *port == req.port)
            .unwrap_or(false);

        if !reuse {
            let conn = WinHttpConnection::new(&self.session, req.hostname, req.port)?;
            self.connection = Some((req.hostname.into(), req.port, conn));
        }

        let (_, _, conn) = self.connection.as_ref().unwrap();
        let request = WinHttpRequest::new(conn, req.method, req.path)?;

        request.send(req.headers, req.body)?;
        request.receive()?;

        self.request = Some(request);

        Ok(())
    }

    pub fn get_status_code(&self) -> Result<u32> {
        if let Some(handle) = &self.request {
            Ok(handle.status_code()?)
        } else {
            Err(Error::from_win32())
        }
    }

    pub fn read_headers<'a>(&'a self, buf: &'a mut [u16]) -> Result<&'a [u16]> {
        if let Some(handle) = &self.request {
            Ok(handle.read_headers(buf)?)
        } else {
            Err(Error::from_win32())
        }
    }

    pub fn read_chunk<'a>(&'a self, buf: &'a mut [u8]) -> Result<usize> {
        if let Some(handle) = &self.request {
            Ok(handle.read_chunk(buf)?)
        } else {
            Err(Error::from_win32())
        }
    }
}

pub struct Request<'a> {
    pub hostname: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,

    /// UTF-16, CRLF-separated, double-null-terminated
    pub headers: Option<&'a [u16]>,

    /// Raw body bytes
    pub body: Option<&'a [u8]>,
}

impl<'a> Request<'a> {
    pub fn new(
        hostname: &'a str,
        port: u16,
        method: &'a str,
        path: &'a str,
        headers: Option<&'a [u16]>,
        body: Option<&'a [u8]>,
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
