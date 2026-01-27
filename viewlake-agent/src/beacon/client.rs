#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use log::info;
use windows::core::{HSTRING, PCWSTR, h};
use winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    connection: Option<WinHttpConnection>,
}

const DEFAULT_AGENT: &HSTRING = h!("DEFAULT_AGENT"); // TODO: randomize user-agent

impl Client {
    /// Initializes WinHttpSession
    pub fn new() -> windows::core::Result<Self> {
        Ok(Client { connection: None })
    }

    /// Sends request.
    pub fn request(&mut self, req: &Request) -> windows::core::Result<Response> {
        let reuse = self
            .connection
            .as_ref()
            .map(|conn| conn.hostname == PCWSTR(req.hostname.as_ptr()) && conn.port == req.port)
            .unwrap_or(false);

        if !reuse {
            info!("creating new connection... user-agent is {}", DEFAULT_AGENT);
            let session = WinHttpSession::new(PCWSTR(DEFAULT_AGENT.as_ptr()))?;
            self.connection = Some(WinHttpConnection::new(
                &session,
                PCWSTR(req.hostname.as_ptr()),
                req.port,
            )?);
        }

        let conn = self.connection.as_ref().unwrap();

        let request =
            WinHttpRequest::new(conn, PCWSTR(req.method.as_ptr()), PCWSTR(req.path.as_ptr()))?;

        request.send(req.headers, req.body)?;

        request.receive()?;

        Ok(Response { handle: request })
    }
}

pub struct Request<'a> {
    pub hostname: &'a HSTRING,
    pub port: u16,
    pub method: &'a HSTRING,
    pub path: &'a HSTRING,

    /// UTF-16, CRLF-separated, double-null-terminated
    pub headers: Option<&'a [u16]>,

    /// Raw body bytes
    pub body: Option<&'a [u8]>,
}

pub struct Response {
    handle: WinHttpRequest,
}

impl Response {
    pub fn get_status_code(&self) -> windows::core::Result<u32> {
        self.handle.status_code()
    }

    pub fn read_headers<'a>(&'a self, buf: &'a mut [u16]) -> windows::core::Result<&'a [u16]> {
        self.handle.read_headers(buf)
    }

    pub fn read_chunk<'a>(&'a self, buf: &'a mut [u8]) -> windows::core::Result<usize> {
        self.handle.read_chunk(buf)
    }
}
