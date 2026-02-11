/// High level API for WinHTTP
mod winhttp;

use log::{debug, info, warn};
use windows::core::{HSTRING, PCWSTR, h};
use winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    connection: Option<WinHttpConnection>,
}

const DEFAULT_AGENT: &HSTRING = h!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36"
);
const MAX_REQUEST_ATTEMPTS: usize = 10;
const RETRY_BACKOFF_MS: u64 = 15000;

impl Client {
    pub fn new() -> windows::core::Result<Self> {
        Ok(Client { connection: None })
    }

    fn with_retry<T, F>(&mut self, mut f: F) -> windows::core::Result<T>
    where
        F: FnMut(&mut Self) -> windows::core::Result<T>,
    {
        let mut last_error = None;

        for attempt in 1..=MAX_REQUEST_ATTEMPTS {
            match f(self) {
                Ok(v) => return Ok(v),
                Err(e) => {
                    warn!(
                        "attempt {}/{} failed: {:?}",
                        attempt, MAX_REQUEST_ATTEMPTS, e
                    );
                    last_error = Some(e);
                    self.connection = None;

                    if attempt < MAX_REQUEST_ATTEMPTS {
                        std::thread::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(windows::core::Error::from_win32))
    }

    fn reuse_connection(&self, req: &Request) -> bool {
        self.connection
            .as_ref()
            .map(|conn| conn.hostname == req.hostname && conn.port == req.port)
            .unwrap_or(false)
    }

    fn ensure_connection(&mut self, req: &Request) -> windows::core::Result<&WinHttpConnection> {
        if self.reuse_connection(req) {
            return Ok(self.connection.as_ref().unwrap());
        }

        info!("creating new connection... user-agent is {}", DEFAULT_AGENT);

        let session = WinHttpSession::new(PCWSTR(DEFAULT_AGENT.as_ptr()))?;
        let connection = WinHttpConnection::new(session, req.hostname.clone(), req.port)?;

        self.connection = Some(connection);
        Ok(self.connection.as_ref().unwrap())
    }

    fn execute_request(conn: &WinHttpConnection, req: &Request) -> windows::core::Result<Response> {
        let handle =
            WinHttpRequest::new(conn, PCWSTR(req.method.as_ptr()), PCWSTR(req.path.as_ptr()))?;

        handle.send(req.headers, req.body)?;
        handle.receive()?;

        debug!("successfully got response!");
        Response::new(handle)
    }

    pub fn request(&mut self, req: Request) -> windows::core::Result<Response> {
        self.with_retry(|client| {
            let conn = client.ensure_connection(&req)?;
            Self::execute_request(conn, &req)
        })
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

#[allow(dead_code)]
pub struct Response {
    pub status_code: u32,
    pub headers: Vec<u16>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(handle: WinHttpRequest) -> windows::core::Result<Self> {
        log::trace!("calling status_code() method on handle");
        let status_code = handle.status_code()?;
        log::debug!("response status code: {status_code}");

        log::trace!("calling read_headers() method on handle");
        let headers = handle.read_headers()?;

        let mut body = Vec::new();
        let mut chunk = vec![0u8; 4096];

        loop {
            let read = handle.read_chunk(&mut chunk)?;
            if read == 0 {
                log::trace!("end of response body reached");
                break;
            }
            body.extend_from_slice(&chunk[..read]);
        }

        Ok(Response {
            status_code,
            headers,
            body,
        })
    }
}
