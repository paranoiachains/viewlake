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
    pub fn request(&mut self, req: Request) -> windows::core::Result<Response> {
        let reuse = self
            .connection
            .as_ref()
            .map(|conn| conn.hostname == req.hostname && conn.port == req.port)
            .unwrap_or(false);

        if !reuse {
            info!("creating new connection... user-agent is {}", DEFAULT_AGENT);
            let session = WinHttpSession::new(PCWSTR(DEFAULT_AGENT.as_ptr()))?;
            self.connection = Some(WinHttpConnection::new(session, req.hostname, req.port)?);
        }

        let conn = self.connection.as_ref().unwrap();

        let request_handle =
            WinHttpRequest::new(conn, PCWSTR(req.method.as_ptr()), PCWSTR(req.path.as_ptr()))?;

        request_handle.send(req.headers, req.body)?;

        request_handle.receive()?;

        Ok(Response::new(request_handle)?)
    }
}

pub struct Request<'a> {
    pub hostname: HSTRING,
    pub port: u16,
    pub method: HSTRING,
    pub path: HSTRING,

    /// UTF-16, CRLF-separated, double-null-terminated
    pub headers: Option<&'a [u16]>,

    /// Raw body bytes
    pub body: Option<&'a [u8]>,
}

pub struct Response {
    pub status_code: u32,
    pub headers: String,
    pub body: String,
}

impl Response {
    pub fn new(handle: WinHttpRequest) -> windows::core::Result<Self> {
        let status_code = handle.status_code()?;

        let mut headers_buf = vec![0u16; 4096];
        let read = handle.read_headers(&mut headers_buf)?;
        headers_buf.truncate(read);

        let mut body = Vec::new();
        let mut chunk = vec![0u8; 4096];

        loop {
            let read = handle.read_chunk(&mut chunk)?;
            if read == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..read]);
        }

        Ok(Response {
            status_code,
            headers: String::from_utf16_lossy(&headers_buf),
            body: String::from_utf8(body)?.to_string(),
        })
    }
}
