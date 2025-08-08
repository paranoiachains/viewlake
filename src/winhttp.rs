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

impl WinHttpSession {
    pub fn new(agent: PCWSTR) -> Result<Self> {
        unsafe {
            let session = WinHttpOpen(
                agent,
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
    pub fn new(session: &WinHttpSession, hostname: PCWSTR) -> Result<Self> {
        unsafe {
            let handle = WinHttpConnect(session.0, hostname, INTERNET_DEFAULT_HTTPS_PORT, 0);

            if handle.is_null() {
                return Err(Error::from_win32());
            } else {
                Ok(Self {
                    handle,
                    hostname: hostname.to_string().expect("hostname is not a valid utf16"),
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
    pub fn new(connection: &WinHttpConnection, method: PCWSTR, path: PCWSTR) -> Result<Self> {
        unsafe {
            let null = PCWSTR::null();
            let null_accept: *const PCWSTR = &null as *const PCWSTR;
            let request = WinHttpOpenRequest(
                connection.handle,
                method,
                path,
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
        headers: Option<Vec<PCWSTR>>,
        body: Option<(*const c_void, u32)>, // pointer + length of body
    ) -> Result<()> {
        let (body_ptr, body_len) = match body {
            Some((ptr, len)) => (Some(ptr), len),
            None => (None, 0),
        };

        if let Some(headers_ptr) = headers {
            self.add_headers(headers_ptr)?;
        }

        unsafe { WinHttpSendRequest(self.0, None, body_ptr, body_len, body_len, 0)? }

        Ok(())
    }

    fn add_headers(&self, headers: Vec<PCWSTR>) -> Result<()> {
        let headers_u16: Vec<u16> = concat_pcwstr(headers);
        let headers_slice: &[u16] = headers_u16.as_slice();
        unsafe { WinHttpAddRequestHeaders(self.0, headers_slice, WINHTTP_ADDREQ_FLAG_ADD) }
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

pub fn concat_pcwstr(headers: Vec<PCWSTR>) -> Vec<u16> {
    let mut combined: Vec<u16> = Vec::new();

    for pcwstr in headers {
        unsafe {
            let mut ptr = pcwstr.as_ptr();
            while !ptr.is_null() && *ptr != 0 {
                combined.push(*ptr);
                ptr = ptr.add(1);
            }
        }
    }
    combined
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use super::*;

    #[test]
    fn create_session() {
        let agent = w!("TestAgent");
        let session = WinHttpSession::new(agent).expect("Failed to create session");
        assert!(!session.0.is_null(), "Session handle is null");
    }

    #[test]
    fn create_connection() {
        let agent = w!("TestAgent");
        let session = WinHttpSession::new(agent).expect("Failed to create session");

        let hostname = w!("www.example.com");
        let connection =
            WinHttpConnection::new(&session, hostname).expect("Failed to create connection");

        assert!(!connection.handle.is_null(), "Connection handle is null");
        assert_eq!(connection.hostname, "www.example.com");
    }

    #[test]
    fn create_request_and_send() {
        let session = WinHttpSession::new(w!("TestAgent")).unwrap();
        let connection = WinHttpConnection::new(&session, w!("www.example.com")).unwrap();
        let request = WinHttpRequest::new(&connection, w!("GET"), w!("/")).unwrap();

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
        let session = WinHttpSession::new(w!("TestAgent")).unwrap();
        let connection = WinHttpConnection::new(&session, w!("www.example.com")).unwrap();
        let request = WinHttpRequest::new(&connection, w!("GET"), w!("/")).unwrap();

        let headers_vec: Vec<PCWSTR> = vec![w!("Header: 1\r\n")];
        let headers_u16: Vec<u16> = concat_pcwstr(headers_vec);
        let headers_slice: &[u16] = headers_u16.as_slice();
        unsafe {
            WinHttpAddRequestHeaders(request.0, headers_slice, WINHTTP_ADDREQ_FLAG_ADD)
                .expect("Failed to add headers to request");

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
    }

    #[test]
    fn send_request_with_body() {
        let session = WinHttpSession::new(w!("TestAgent")).unwrap();
        let connection = WinHttpConnection::new(&session, w!("www.example.com")).unwrap();
        let request = WinHttpRequest::new(&connection, w!("GET"), w!("/")).unwrap();

        let body = "asd";
        let body_ptr = body.as_bytes().as_ptr() as *const c_void;
        let body_len = body.as_bytes().len() as u32;

        request.send(None, Some((body_ptr, body_len))).unwrap();
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
