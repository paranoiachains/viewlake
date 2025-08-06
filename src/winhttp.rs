use std::os::raw::c_void;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Networking::WinHttp::*;
use windows::core::*;

// Steps with WinHTTP:
// Open a session with WinHttpOpen
// Connect to a server with WinHttpConnect
// Open a request handle with WinHttpOpenRequest
// Send the request with WinHttpSendRequest
// Receive the response with WinHttpReceiveResponse
// Read the response using WinHttpReadData

pub type HINTERNET = *mut c_void;

pub struct WinHttpSession(pub HINTERNET);

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
                Err(GetLastError().unwrap_err())
            } else {
                Ok(Self(session))
            }
        }
    }
}

impl Into<HINTERNET> for WinHttpSession {
    fn into(self) -> HINTERNET {
        self.0
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
                Err(GetLastError().unwrap_err())
            } else {
                Ok(Self {
                    handle,
                    hostname: hostname.to_string().unwrap(),
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
    pub fn new(
        connection: &WinHttpConnection,
        method: PCWSTR,
        path: PCWSTR,
        accept_types: *const PCWSTR,
    ) -> Result<Self> {
        unsafe {
            let request = WinHttpOpenRequest(
                connection.handle,
                method,
                path,
                PCWSTR::null(), // HTTP version (1.1)
                PCWSTR::null(), // Referer
                accept_types,
                WINHTTP_FLAG_SECURE,
            );

            if request.is_null() {
                return Err(GetLastError().unwrap_err());
            }
            Ok(Self(request))
        }
    }

    pub fn send(
        &self,
        headers: Option<&[u16]>,
        body: Option<(*const c_void, u32)>, // pointer + length of body
    ) -> Result<()> {
        unsafe {
            let headers_ptr = match headers {
                Some(h) => Some(h),
                None => None,
            };

            let (body_ptr, body_len) = match body {
                Some((ptr, len)) => (Some(ptr), len),
                None => (None, 0),
            };

            let ok = WinHttpSendRequest(self.0, headers_ptr, body_ptr, body_len, body_len, 0);
            if let Err(e) = ok {
                return Err(e);
            }

            Ok(())
        }
    }

    pub fn receive(&self) -> Result<()> {
        unsafe { WinHttpReceiveResponse(self.0, std::ptr::null_mut() as *mut c_void) }
    }

    pub fn read_response(&self, buf: *mut c_void, buf_len: u32) -> Result<()> {
        unsafe {
            let bytes_read: *mut u32 = 0 as *mut u32;

            let ok = WinHttpReadData(self.0, buf, buf_len, bytes_read);

            if let Err(e) = ok {
                return Err(e);
            }

            Ok(())
        }
    }
}

impl Into<HINTERNET> for WinHttpRequest {
    fn into(self) -> HINTERNET {
        self.0
    }
}

impl Drop for WinHttpRequest {
    fn drop(&mut self) {
        unsafe { WinHttpCloseHandle(self.0).unwrap() }
    }
}
