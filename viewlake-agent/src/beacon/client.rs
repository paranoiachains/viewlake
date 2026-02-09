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

    pub fn request(&mut self, req: Request) -> windows::core::Result<Response> {
        let mut last_error = None;

        for attempt in 1..=MAX_REQUEST_ATTEMPTS {
            let reuse = self
                .connection
                .as_ref()
                .map(|conn| conn.hostname == req.hostname && conn.port == req.port)
                .unwrap_or(false);

            if !reuse {
                info!("creating new connection... user-agent is {}", DEFAULT_AGENT);
                let session = match WinHttpSession::new(PCWSTR(DEFAULT_AGENT.as_ptr())) {
                    Ok(session) => session,
                    Err(err) => {
                        warn!(
                            "attempt {}/{}: failed to initialize WinHTTP session: {:?}",
                            attempt, MAX_REQUEST_ATTEMPTS, err
                        );
                        last_error = Some(err);
                        self.connection = None;
                        if attempt < MAX_REQUEST_ATTEMPTS {
                            std::thread::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS));
                        }
                        continue;
                    }
                };

                match WinHttpConnection::new(session, req.hostname.clone(), req.port) {
                    Ok(connection) => {
                        self.connection = Some(connection);
                    }
                    Err(err) => {
                        warn!(
                            "attempt {}/{}: failed to connect to {}:{}: {:?}",
                            attempt, MAX_REQUEST_ATTEMPTS, req.hostname, req.port, err
                        );
                        last_error = Some(err);
                        self.connection = None;
                        if attempt < MAX_REQUEST_ATTEMPTS {
                            std::thread::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS));
                        }
                        continue;
                    }
                }
            }

            let conn = self.connection.as_ref().unwrap();

            let request_handle = match WinHttpRequest::new(
                conn,
                PCWSTR(req.method.as_ptr()),
                PCWSTR(req.path.as_ptr()),
            ) {
                Ok(handle) => handle,
                Err(e) => {
                    warn!(
                        "attempt {}/{}: failed to open request: {:?}",
                        attempt, MAX_REQUEST_ATTEMPTS, e
                    );
                    last_error = Some(e);

                    self.connection = None;

                    if attempt < MAX_REQUEST_ATTEMPTS {
                        std::thread::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS));
                    }
                    continue;
                }
            };

            if let Err(e) = request_handle.send(req.headers, req.body) {
                warn!(
                    "attempt {}/{}: failed to send request: {:?}",
                    attempt, MAX_REQUEST_ATTEMPTS, e
                );
                last_error = Some(e);

                self.connection = None;
                if attempt < MAX_REQUEST_ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS));
                }
                continue;
            };

            if let Err(e) = request_handle.receive() {
                warn!(
                    "attempt {}/{}: failed to receive response: {:?}",
                    attempt, MAX_REQUEST_ATTEMPTS, e
                );
                last_error = Some(e);

                self.connection = None;

                if attempt < MAX_REQUEST_ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS));
                }
                continue;
            }
            debug!("successfully got response!");
            return Response::new(request_handle);
        }
        Err(last_error.unwrap_or(windows::core::Error::from_win32()))
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
            body: String::from_utf8_lossy(&body).to_string(),
        })
    }
}
