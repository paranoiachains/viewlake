/// Low-level API for WinHTTP
use core::ffi::c_void;
use log::{debug, error};
use windows::Win32::Networking::WinHttp::{self, WinHttpSetOption};
use windows::core::{Error, HSTRING, PCWSTR};

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
            None => Err(Error::from_win32()),
        }
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
                return Err(Error::from_win32());
            } else {
                Ok(Self {
                    handle: WinHttpHandle(Some(session)),
                })
            }
        }
    }
}

/// WinHttpConnection uses WinHttpSession to invoke connection and stores handle to connection +
/// target hostname + session handle pointer so it is not dropped afterwards
pub struct WinHttpConnection {
    pub handle: WinHttpHandle,
    pub hostname: HSTRING,
    pub port: u16,
    #[allow(dead_code)] // session is owned, pointer preserved
    session: WinHttpSession,
}

impl WinHttpConnection {
    pub fn new(
        session: WinHttpSession,
        hostname: HSTRING,
        port: u16,
    ) -> windows::core::Result<Self> {
        debug!("initializing winhttpconnection...");
        unsafe {
            let handle = WinHttp::WinHttpConnect(
                session.handle.ok_or_else()?,
                PCWSTR(hostname.as_ptr()),
                port,
                0,
            );

            if handle.is_null() {
                error!("winhttpconnect returned null");
                return Err(Error::from_win32());
            } else {
                Ok(Self {
                    handle: WinHttpHandle(Some(handle)),
                    hostname,
                    port,
                    session,
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

            if request.is_null() {
                error!("request handle is null");
                return Err(Error::from_win32());
            }

            debug!("request handle opened successfully");

            Self::set_winhttp_options(request)?;

            debug!("http options set");

            Ok(Self {
                handle: WinHttpHandle(Some(request)),
            })
        }
    }

    fn set_winhttp_options(handle: *mut c_void) -> windows::core::Result<()> {
        let flags: u32 = WinHttp::SECURITY_FLAG_IGNORE_UNKNOWN_CA
            | WinHttp::SECURITY_FLAG_IGNORE_CERT_CN_INVALID
            | WinHttp::SECURITY_FLAG_IGNORE_CERT_DATE_INVALID;

        unsafe {
            WinHttpSetOption(
                Some(handle),
                WinHttp::WINHTTP_OPTION_SECURITY_FLAGS,
                Some(std::slice::from_raw_parts(
                    &flags as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            )
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
            WinHttp::WinHttpSendRequest(
                self.handle.ok_or_else()?,
                None,
                body_ptr,
                body_len,
                body_len,
                0,
            )?;
        }

        debug!("request sent");

        Ok(())
    }

    /// Helper function to add headers to request
    fn add_headers(&self, headers: &[u16]) -> windows::core::Result<()> {
        unsafe {
            WinHttp::WinHttpAddRequestHeaders(
                self.handle.ok_or_else()?,
                headers,
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

    pub fn read_headers(&self) -> windows::core::Result<Vec<u16>> {
        let mut size_bytes = self.get_resp_headers_size()?;

        unsafe {
            let mut buf = vec![0u8; size_bytes as usize];

            WinHttp::WinHttpQueryHeaders(
                self.handle.ok_or_else()?,
                WinHttp::WINHTTP_QUERY_RAW_HEADERS,
                PCWSTR::null(),
                Some(buf.as_mut_ptr().cast()),
                &mut size_bytes,
                std::ptr::null_mut(),
            )?;

            let headers = buf
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();

            Ok(headers)
        }
    }

    fn get_resp_headers_size(&self) -> windows::core::Result<u32> {
        let mut size_bytes: u32 = 0;

        unsafe {
            let res = WinHttp::WinHttpQueryHeaders(
                self.handle.ok_or_else()?,
                WinHttp::WINHTTP_QUERY_RAW_HEADERS,
                PCWSTR::null(),
                None,
                &mut size_bytes,
                std::ptr::null_mut(),
            );

            if res.is_ok()
                || Error::from_win32().code()
                    != windows::core::HRESULT::from_win32(
                        windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER.0,
                    )
            {
                return Err(Error::from_win32());
            }
        }

        Ok(size_bytes)
    }
}
