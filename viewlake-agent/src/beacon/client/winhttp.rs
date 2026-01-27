/// Low-level API for WinHTTP
use core::ffi::c_void;
use log::{debug, error};
use windows::Win32::Networking::WinHttp::{self, WinHttpSetOption};
use windows::core::{Error, HRESULT, HSTRING, PCWSTR};

// Steps with WinHTTP:
// Open a session with WinHttpOpen
// Connect to a server with WinHttpConnect
// Open a request handle with WinHttpOpenRequest
// Send the request with WinHttpSendRequest
// Receive the response with WinHttpReceiveResponse
// Read the response using WinHttpReadData

/// Return type of almost every WinAPI func used here
type HINTERNET = *mut c_void;

/// WinAPI HINTERNET wrapper, which implements Drop trait
pub struct WinHttpHandle(Option<HINTERNET>);

impl WinHttpHandle {
    /// Helper methods to check if handle is not null
    pub fn ok_or_else(&self) -> windows::core::Result<HINTERNET> {
        match self.0 {
            Some(h) => Ok(h),
            None => Err(Error::new(
                HRESULT(1),
                HSTRING::from("winhttphandle is null"),
            )),
        }
    }

    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }
}

impl Drop for WinHttpHandle {
    /// Calls WinHttpCloseHandle on drop
    fn drop(&mut self) {
        if let Some(h) = self.0.take() {
            unsafe {
                WinHttp::WinHttpCloseHandle(h).ok();
            }
        }
    }
}

/// WinHttpSession abstraction
pub struct WinHttpSession {
    handle: WinHttpHandle,
}

impl WinHttpSession {
    /// Returns WinHttpSession with the given user-agent
    pub fn new(agent: PCWSTR) -> windows::core::Result<Self> {
        debug!("initializing  winhttpsession...");
        unsafe {
            let session = WinHttp::WinHttpOpen(
                agent,
                WinHttp::WINHTTP_ACCESS_TYPE_NO_PROXY,
                PCWSTR::null(), // WINHTTP_NO_PROXY_NAME
                PCWSTR::null(), // WINHTTP_NO_PROXY_BYPASS
                WinHttp::WINHTTP_FLAG_SECURE_DEFAULTS,
            );

            if session.is_null() {
                return Err(Error::new(
                    HRESULT(1),
                    HSTRING::from("winhttpsession is null"),
                ));
            } else {
                Ok(Self {
                    handle: WinHttpHandle(Some(session)),
                })
            }
        }
    }
}

/// WinHttpConnection uses WinHttpSession to invoke session and stores handle to connection +
/// target hostname
pub struct WinHttpConnection {
    pub handle: WinHttpHandle,
    pub hostname: PCWSTR,
    pub port: u16,
}

impl WinHttpConnection {
    pub fn new(
        session: WinHttpSession,
        hostname: PCWSTR,
        port: u16,
    ) -> windows::core::Result<Self> {
        debug!("initializing winhttpconnection...");
        unsafe {
            let handle = WinHttp::WinHttpConnect(session.handle.ok_or_else()?, hostname, port, 0);

            if handle.is_null() {
                error!("winhttpconnect returned null");
                return Err(Error::new(
                    HRESULT(1),
                    HSTRING::from("winhttpconnection is null"),
                ));
            } else {
                Ok(Self {
                    handle: WinHttpHandle(Some(handle)),
                    hostname,
                    port,
                })
            }
        }
    }
}

/// Stores handle to HTTP request
pub struct WinHttpRequest {
    handle: WinHttpHandle,
}

impl WinHttpRequest {
    pub fn new(
        connection: &WinHttpConnection,
        method: PCWSTR,
        path: PCWSTR,
    ) -> windows::core::Result<Self> {
        debug!("initializing winhttprequest...");

        unsafe {
            let request = WinHttp::WinHttpOpenRequest(
                connection.handle.ok_or_else()?,
                method,
                path,
                PCWSTR::null(),
                PCWSTR::null(),
                std::ptr::null(),
                WinHttp::WINHTTP_FLAG_SECURE,
            );

            debug!("request handle opened");

            if request.is_null() {
                error!("request handle is null");
                return Err(Error::new(
                    HRESULT(1),
                    HSTRING::from("winhttpopenrequest returned null"),
                ));
            }

            // === TLS flags ===
            let flags: u32 = WinHttp::SECURITY_FLAG_IGNORE_UNKNOWN_CA
                | WinHttp::SECURITY_FLAG_IGNORE_CERT_CN_INVALID
                | WinHttp::SECURITY_FLAG_IGNORE_CERT_DATE_INVALID;

            WinHttpSetOption(
                Some(request),
                WinHttp::WINHTTP_OPTION_SECURITY_FLAGS,
                Some(std::slice::from_raw_parts(
                    &flags as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            )?;

            debug!("http options set");

            Ok(Self {
                handle: WinHttpHandle(Some(request)),
            })
        }
    }

    /// Sends HTTP request with given headers and body
    /// `headers` must be UTF-16, CRLF-separated, and double-null terminated.

    pub fn send(&self, headers: Option<&[u16]>, body: Option<&[u8]>) -> windows::core::Result<()> {
        debug!("sending http request...");
        let (body_ptr, body_len) = body
            .map(|b| (Some(b.as_ptr() as *const c_void), b.len() as u32))
            .unwrap_or((None, 0));

        if let Some(h) = headers {
            self.add_headers(h)?;
        }

        unsafe {
            if let Err(e) = WinHttp::WinHttpSendRequest(
                self.handle.ok_or_else()?,
                None,
                body_ptr,
                body_len,
                body_len,
                0,
            ) {
                error!("got error [winhttpsendrequest]: {}", e);
                return Err(e);
            }
        }

        debug!("request sent");

        Ok(())
    }

    /// Helper function to add headers to request
    fn add_headers(&self, headers: &[u16]) -> windows::core::Result<()> {
        unsafe {
            WinHttp::WinHttpAddRequestHeaders(
                self.handle.ok_or_else()?,
                &headers,
                WinHttp::WINHTTP_ADDREQ_FLAG_ADD,
            )?;
        }

        debug!("headers added");
        Ok(())
    }

    /// Receive response
    pub fn receive(&self) -> windows::core::Result<()> {
        unsafe {
            WinHttp::WinHttpReceiveResponse(
                self.handle.ok_or_else()?,
                std::ptr::null_mut() as *mut c_void,
            )
        }
    }

    /// Read response to buffer
    pub fn read_chunk(&self, buf: &mut [u8]) -> windows::core::Result<usize> {
        unsafe {
            let mut bytes_read: u32 = 0;
            WinHttp::WinHttpReadData(
                self.handle.ok_or_else()?,
                buf.as_mut_ptr() as *mut c_void,
                buf.len().try_into()?,
                &mut bytes_read,
            )?;

            Ok(bytes_read as usize)
        }
    }

    pub fn status_code(&self) -> windows::core::Result<u32> {
        unsafe {
            let mut code: u32 = 0;
            let mut length = std::mem::size_of::<u32>() as u32;

            WinHttp::WinHttpQueryHeaders(
                self.handle.ok_or_else()?,
                WinHttp::WINHTTP_QUERY_STATUS_CODE | WinHttp::WINHTTP_QUERY_FLAG_NUMBER,
                PCWSTR::null(),
                Some(&mut code as *mut u32 as *mut _),
                &mut length,
                std::ptr::null_mut(),
            )?;

            Ok(code)
        }
    }

    pub fn read_headers<'a>(&self, buf: &'a mut [u16]) -> windows::core::Result<&'a [u16]> {
        unsafe {
            let mut size_bytes: u32 = 0;

            // 1. Query required size (in BYTES)
            WinHttp::WinHttpQueryHeaders(
                self.handle.ok_or_else()?,
                WinHttp::WINHTTP_QUERY_RAW_HEADERS,
                PCWSTR::null(),
                None,
                &mut size_bytes,
                std::ptr::null_mut(),
            )
            .ok(); // expected to fail with ERROR_INSUFFICIENT_BUFFER

            let required_u16 = (size_bytes as usize) / 2;

            if buf.len() < required_u16 {
                return Err(Error::new(
                    HRESULT(1),
                    HSTRING::from("exceeded buffer size"),
                ));
            }

            // 2. Fetch headers into caller buffer
            let mut size_bytes = (buf.len() * 2) as u32;

            WinHttp::WinHttpQueryHeaders(
                self.handle.ok_or_else()?,
                WinHttp::WINHTTP_QUERY_RAW_HEADERS,
                PCWSTR::null(),
                Some(buf.as_mut_ptr() as *mut c_void),
                &mut size_bytes,
                std::ptr::null_mut(),
            )?;

            let written_u16 = (size_bytes as usize) / 2;

            Ok(&buf[..written_u16])
        }
    }
}
