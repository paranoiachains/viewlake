use std::os::raw::c_void;
use windows::Win32::Networking::WinHttp::*;
use windows::core::*;

// Steps with WinHTTP:
// Open a session with WinHttpOpen
// Connect to a server with WinHttpConnect
// Open a request handle with WinHttpOpenRequest
// Send the request with WinHttpSendRequest
// Receive the response with WinHttpReceiveResponse
// Read the response using WinHttpReadData

type HINTERNET = *mut c_void;

pub struct WinHttpSession(HINTERNET);

fn to_pcwstr(s: &String) -> PCWSTR {
    let mut s_utf16: Vec<u16> = s.encode_utf16().collect();
    s_utf16.push(0);
    println!("MY STRING: {:?}", s_utf16);
    PCWSTR::from_raw(s_utf16.as_ptr())
}

impl WinHttpSession {
    pub fn new(agent: String) -> Result<Self> {
        unsafe {
            let session = WinHttpOpen(
                to_pcwstr(&agent),
                WINHTTP_ACCESS_TYPE_NO_PROXY,
                PCWSTR::null(), // WINHTTP_NO_PROXY_NAME
                PCWSTR::null(), // WINHTTP_NO_PROXY_BYPASS
                WINHTTP_FLAG_SECURE_DEFAULTS,
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
        unsafe { WinHttpCloseHandle(self.0).unwrap() }
    }
}

pub struct WinHttpConnection {
    pub handle: HINTERNET,
    pub hostname: String,
}

impl WinHttpConnection {
    pub fn new(session: &WinHttpSession, hostname: String) -> Result<Self> {
        unsafe {
            let handle = WinHttpConnect(
                session.0,
                to_pcwstr(&hostname),
                INTERNET_DEFAULT_HTTPS_PORT,
                0,
            );

            if handle.is_null() {
                return Err(Error::from_win32());
            } else {
                Ok(Self {
                    handle,
                    hostname: hostname,
                })
            }
        }
    }
}

impl Drop for WinHttpConnection {
    fn drop(&mut self) {
        unsafe { WinHttpCloseHandle(self.handle).unwrap() }
    }
}

pub struct WinHttpRequest(HINTERNET);

impl WinHttpRequest {
    pub fn new(connection: &WinHttpConnection, method: String, path: String) -> Result<Self> {
        unsafe {
            let null = PCWSTR::null();
            let null_accept: *const PCWSTR = &null as *const PCWSTR;
            let request = WinHttpOpenRequest(
                connection.handle,
                to_pcwstr(&method),
                to_pcwstr(&path),
                PCWSTR::null(), // HTTP version (1.1)
                PCWSTR::null(), // Referer
                null_accept,    // Accept
                WINHTTP_FLAG_SECURE,
            );

            if request.is_null() {
                return Err(Error::from_win32());
            }
            Ok(Self(request))
        }
    }

    pub fn send(
        &self,
        headers: Option<Vec<String>>,
        body: Option<String>, // pointer + length of body
    ) -> Result<()> {
        let body_option = body.map(|body_str| {
            let body_ptr = body_str.as_bytes().as_ptr() as *const c_void;
            let body_len = body_str.as_bytes().len() as u32;
            (body_ptr, body_len)
        });

        let (body_ptr, body_len) = match body_option {
            Some((ptr, len)) => (Some(ptr), len),
            None => (None, 0),
        };

        if let Some(headers_ptr) = headers {
            self.add_headers(headers_ptr)?;
        }

        unsafe { WinHttpSendRequest(self.0, None, body_ptr, body_len, body_len, 0)? }

        Ok(())
    }

    fn add_headers(&self, headers: Vec<String>) -> Result<()> {
        let mut combined: Vec<u16> = Vec::new();

        for (i, header) in headers.iter().enumerate() {
            combined.extend(header.encode_utf16());

            if i != headers.len() - 1 {
                combined.push(b'\r' as u16);
                combined.push(b'\n' as u16);
            }
        }

        combined.push(0);

        unsafe { WinHttpAddRequestHeaders(self.0, &combined, WINHTTP_ADDREQ_FLAG_ADD) }
    }

    pub fn receive(&self) -> Result<()> {
        unsafe { WinHttpReceiveResponse(self.0, std::ptr::null_mut() as *mut c_void) }
    }

    pub fn read(&self, buf: *mut c_void, buf_len: u32) -> Result<()> {
        unsafe {
            let mut bytes_read: u32 = 0;

            WinHttpReadData(self.0, buf, buf_len, &mut bytes_read)?;

            Ok(())
        }
    }
}

impl Drop for WinHttpRequest {
    fn drop(&mut self) {
        unsafe { WinHttpCloseHandle(self.0).unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use super::*;

    #[test]
    fn create_session() {
        let agent = "TestAgent".to_string();
        let session = WinHttpSession::new(agent).expect("Failed to create session");
        assert!(!session.0.is_null(), "Session handle is null");
    }

    #[test]
    fn create_connection() {
        let agent = "TestAgent".to_string();
        let session = WinHttpSession::new(agent).expect("Failed to create session");

        let hostname = "www.example.com".to_string();
        let connection =
            WinHttpConnection::new(&session, hostname).expect("Failed to create connection");

        assert!(!connection.handle.is_null(), "Connection handle is null");
        assert_eq!(connection.hostname, "www.example.com".to_string());
    }

    #[test]
    fn create_request_and_send() {
        let session = WinHttpSession::new("TestAgent".to_string()).unwrap();
        let connection = WinHttpConnection::new(&session, "www.example.com".to_string()).unwrap();
        let request = WinHttpRequest::new(&connection, "GET".to_string(), "/".to_string()).unwrap();

        request.send(None, None).unwrap();
        request.receive().unwrap();

        let mut buf = [0u8; 4096];
        request
            .read(buf.as_mut_ptr() as *mut _, buf.len() as u32)
            .unwrap();
        if let Ok(text) = std::str::from_utf8(&buf) {
            println!("{text}");
        } else {
            io::stdout().write_all(&buf).unwrap();
        }
    }

    #[test]
    fn send_request_with_headers() {
        let session = WinHttpSession::new("TestAgent".to_string()).unwrap();
        let connection = WinHttpConnection::new(&session, "www.example.com".to_string()).unwrap();
        let request = WinHttpRequest::new(&connection, "GET".to_string(), "/".to_string()).unwrap();

        let headers_vec: Vec<String> = vec![String::from("Header: 1")];
        request.send(Some(headers_vec), None).unwrap();
        request.receive().unwrap();

        let mut buf = [0u8; 4096];
        request
            .read(buf.as_mut_ptr() as *mut _, buf.len() as u32)
            .unwrap();
        if let Ok(text) = std::str::from_utf8(&buf) {
            println!("{text}");
        } else {
            io::stdout().write_all(&buf).unwrap();
        }
    }

    #[test]
    fn send_request_with_body() {
        let session = WinHttpSession::new("TestAgent".to_string()).unwrap();
        let connection = WinHttpConnection::new(&session, "www.example.com".to_string()).unwrap();
        let request = WinHttpRequest::new(&connection, "GET".to_string(), "/".to_string()).unwrap();

        let body = "asd".to_string();

        request.send(None, Some(body)).unwrap();
        request.receive().unwrap();

        let mut buf = [0u8; 4096];
        request
            .read(buf.as_mut_ptr() as *mut _, buf.len() as u32)
            .unwrap();
        if let Ok(text) = std::str::from_utf8(&buf) {
            println!("{text}");
        } else {
            io::stdout().write_all(&buf).unwrap();
        }
    }
}
