pub struct Client {}

use std::os::raw::c_void;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Foundation::*;
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

pub struct WinHttpConnection(HINTERNET);

impl WinHttpConnection {
    pub fn new(session: WinHttpSession, hostname: PCWSTR) -> Result<Self> {
        unsafe {
            let session = WinHttpConnect(session.into(), hostname, INTERNET_DEFAULT_HTTPS_PORT, 0);

            if session.is_null() {
                Err(GetLastError().unwrap_err())
            } else {
                Ok(Self(session))
            }
        }
    }
}
impl Into<HINTERNET> for WinHttpConnection {
    fn into(self) -> HINTERNET {
        self.0
    }
}

pub struct WinHttpRequest(HINTERNET);

impl WinHttpRequest {
    pub fn new(
        connection: WinHttpConnection,
        method: PCWSTR,
        path: PCWSTR,
        accept_types: *const PCWSTR,
        headers: Option<&[u16]>,
    ) -> Result<Self> {
        unsafe {
            let request = WinHttpOpenRequest(
                connection.into(),
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

            if let Some(headers) = headers {
                if let Err(e) = WinHttpAddRequestHeaders(request, headers, WINHTTP_ADDREQ_FLAG_ADD)
                {
                    return Err(e); // kinda cringe but whatever
                }
            };
            Ok(Self(request))
        }
    }

    pub fn send(&self, body: Option<*const c_void>) -> Result<()> {}
}

impl Into<HINTERNET> for WinHttpRequest {
    fn into(self) -> HINTERNET {
        self.0
    }
}
