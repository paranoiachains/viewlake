use std::os::raw::c_void;
use widestring;
use windows::Win32::Networking::WinHttp;
use windows::core::{Error, PCWSTR};

// Steps with WinHTTP:
// Open a session with WinHttpOpen
// Connect to a server with WinHttpConnect
// Open a request handle with WinHttpOpenRequest
// Send the request with WinHttpSendRequest
// Receive the response with WinHttpReceiveResponse
// Read the response using WinHttpReadData

type HINTERNET = *mut c_void;

pub(super) struct WinHttpSession(HINTERNET);

fn to_wide(s: &str) -> Utf16String {
    let mut u16 = widestring::Utf16String::from_str(s);
    u16.push('\0');
    u16
}

impl WinHttpSession {
    pub(super) fn new(agent: &str) -> Result<Self> {
        let agent_wide = to_wide(agent);
        unsafe {
            let session = WinHttp::WinHttpOpen(
                PCWSTR(agent_wide.as_ptr()),
                WinHttp::WINHTTP_ACCESS_TYPE_NO_PROXY,
                PCWSTR::null(), // WINHTTP_NO_PROXY_NAME
                PCWSTR::null(), // WINHTTP_NO_PROXY_BYPASS
                WinHttp::WINHTTP_FLAG_SECURE_DEFAULTS,
            );

            if session.is_null() {
                return Err(Error::from_win32());
            } else {
                Ok(Self(session))
            }
        }
    }
}

impl Drop for WinHttpSession {
    fn drop(&mut self) {
        unsafe { WinHttp::WinHttpCloseHandle(self.0).unwrap() }
    }
}

pub(super) struct WinHttpConnection {
    pub(super) handle: HINTERNET,
    pub(super) hostname: String,
}

impl WinHttpConnection {
    pub(super) fn new(session: &WinHttpSession, hostname: &str, port: u16) -> Result<Self> {
        let hostname_wide = to_wide(hostname);
        unsafe {
            let handle =
                WinHttp::WinHttpConnect(session.0, PCWSTR(hostname_wide.as_ptr()), port, 0);

            if handle.is_null() {
                return Err(Error::from_win32());
            } else {
                Ok(Self {
                    handle,
                    hostname: hostname.to_string(),
                })
            }
        }
    }
}

impl Drop for WinHttpConnection {
    fn drop(&mut self) {
        unsafe { WinHttp::WinHttpCloseHandle(self.handle).unwrap() }
    }
}

pub(super) struct WinHttpRequest(HINTERNET);

impl WinHttpRequest {
    pub(super) fn new(connection: &WinHttpConnection, method: &str, path: &str) -> Result<Self> {
        let method_wide = to_wide(method);
        let path_wide = to_wide(path);
        let null = PCWSTR::null();
        let null_accept: *const PCWSTR = &null as *const PCWSTR;

        unsafe {
            let request = WinHttpOpenRequest(
                connection.handle,
                PCWSTR(method_wide.as_ptr()),
                PCWSTR(path_wide.as_ptr()),
                PCWSTR::null(), // HTTP version (1.1)
                PCWSTR::null(), // Referer
                null_accept,    // Accept
                WinHttp::WINHTTP_FLAG_SECURE,
            );

            if request.is_null() {
                return Err(Error::from_win32());
            }

            Ok(Self(request))
        }
    }

    pub(super) fn send(
        &self,
        headers: Option<Vec<&str>>,
        body: Option<&str>, // pointer + length of body
    ) -> Result<()> {
        let body_bytes_opt = body.as_ref().map(|b| b.as_bytes());

        let (body_ptr, body_len) = if let Some(bytes) = body_bytes_opt {
            (Some(bytes.as_ptr() as *const c_void), bytes.len() as u32)
        } else {
            (None, 0 as u32)
        };

        if let Some(headers_ptr) = headers {
            self.add_headers(headers_ptr)?;
        }

        unsafe { WinHttp::WinHttpSendRequest(self.0, None, body_ptr, body_len, body_len, 0)? }

        Ok(())
    }

    fn add_headers(&self, headers: Vec<&str>) -> Result<()> {
        let mut combined: Vec<u16> = Vec::new();

        for (i, header) in headers.iter().enumerate() {
            combined.extend(header.encode_utf16());

            if i != headers.len() - 1 {
                combined.push(b'\r' as u16);
                combined.push(b'\n' as u16);
            }
        }

        combined.push(0);

        unsafe {
            WinHttp::WinHttpAddRequestHeaders(self.0, &combined, WinHttp::WINHTTP_ADDREQ_FLAG_ADD)
        }
    }

    pub(super) fn receive(&self) -> Result<()> {
        unsafe { WinHttp::WinHttpReceiveResponse(self.0, std::ptr::null_mut() as *mut c_void) }
    }

    pub(super) fn read(&self, buf: *mut c_void, buf_len: u32) -> Result<()> {
        unsafe {
            let mut bytes_read: u32 = 0;

            WinHttp::WinHttpReadData(self.0, buf, buf_len, &mut bytes_read)?;

            Ok(())
        }
    }
}

impl Drop for WinHttpRequest {
    fn drop(&mut self) {
        unsafe { WinHttp::WinHttpCloseHandle(self.0).unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use super::*;

    #[test]
    fn create_session() {
        let agent = "TestAgent";
        let session = WinHttpSession::new(agent).expect("Failed to create session");
        assert!(!session.0.is_null(), "Session handle is null");
    }

    #[test]
    fn create_connection() {
        let agent = "TestAgent";
        let session = WinHttpSession::new(agent).expect("Failed to create session");

        let hostname = "www.example.com";
        let port = 443;
        let connection =
            WinHttpConnection::new(&session, hostname, port).expect("Failed to create connection");

        assert!(!connection.handle.is_null(), "Connection handle is null");
        assert_eq!(connection.hostname, "www.example.com");
    }

    #[test]
    fn create_request_and_send() {
        let session = WinHttpSession::new("TestAgent").unwrap();
        let connection = WinHttpConnection::new(&session, "www.example.com", 443).unwrap();
        let request = WinHttpRequest::new(&connection, "GET", "/").unwrap();

        request.send(None, None).unwrap();
        request.receive().unwrap();

        let mut buf = [0u8; 4096];
        request
            .read(buf.as_mut_ptr() as *mut _, buf.len() as u32)
            .unwrap();

        assert!(!buf.is_empty());
    }

    #[test]
    fn send_request_with_headers() {
        let session = WinHttpSession::new("TestAgent").unwrap();
        let connection = WinHttpConnection::new(&session, "www.example.com", 443).unwrap();
        let request = WinHttpRequest::new(&connection, "GET", "/").unwrap();

        let headers_vec: Vec<&str> = vec!["Header: 1"];
        request.send(Some(headers_vec), None).unwrap();
        request.receive().unwrap();

        let mut buf = [0u8; 4096];
        assert!(!buf.is_empty())
    }

    #[test]
    fn send_request_with_body() {
        let session = WinHttpSession::new("TestAgent").unwrap();
        let connection = WinHttpConnection::new(&session, "www.example.com", 443).unwrap();
        let request = WinHttpRequest::new(&connection, "GET", "/").unwrap();

        let body = "Example body";

        request.send(None, Some(body)).unwrap();
        request.receive().unwrap();

        let mut buf = [0u8; 4096];
        request
            .read(buf.as_mut_ptr() as *mut _, buf.len() as u32)
            .unwrap();

        assert!(!buf.is_empty())
    }
}
