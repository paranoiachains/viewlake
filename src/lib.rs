#![no_std]
#![allow(unused_imports)]

use core::any::{Any, TypeId};
use core::ffi::c_void;
use core::panic::PanicInfo;
use core::ptr::null;

use windows_sys::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows_sys::Win32::Networking::WinHttp::*;
use windows_sys::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE, WriteConsoleA};
use windows_sys::Win32::System::Threading::ExitProcess;

use crate::utf::to_utf16;
pub mod utf;

#[panic_handler]
pub fn panic(_: &PanicInfo<'_>) -> ! {
    unsafe {
        log_to_console("panic occured\n");
        ExitProcess(1);
    }
}

#[cfg(feature = "logging")]
pub fn log_to_console(s: &str) {
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        let _ = WriteConsoleA(
            console,
            s.as_ptr() as *const c_void,
            s.len() as u32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
}

#[cfg(not(feature = "logging"))]
pub fn log_to_console(_s: &str) {}

pub struct HttpRequest<'a> {
    pub hostname: &'a [u16],
    pub path: &'a [u16],
    pub method: &'a [u16],
    pub headers: &'a [&'a [u16]],
    pub body: Option<&'a [u8]>,
    pub accept: &'a [u16],
}

impl<'a> HttpRequest<'a> {
    pub fn new(buffers: &'a utf::Utf16Buffers, header_refs: &'a mut [&'a [u16]]) -> Self {
        for i in 0..buffers.headers_count {
            header_refs[i] = &buffers.headers[i][..buffers.headers_len[i]];
        }
        let body_bytes: Option<&'a [u8]> = if let Some((ref arr, len)) = buffers.body {
            // SAFETY: arr is &[u16], reinterpret as &[u8]
            Some(unsafe { core::slice::from_raw_parts(arr.as_ptr() as *const u8, len * 2) })
        } else {
            None
        };

        HttpRequest {
            hostname: &buffers.hostname[..buffers.hostname_len],
            path: &buffers.path[..buffers.path_len],
            method: &buffers.method[..buffers.method_len],
            headers: &header_refs[..buffers.headers_count],
            body: body_bytes,
            accept: &buffers.accept[..buffers.accept_len],
        }
    }
}

pub struct Session<'a> {
    pub session: *mut c_void,
    pub connection_handle: *mut c_void,
    pub request_handle: *mut c_void,
    pub http_request: HttpRequest<'a>,
}

impl<'a> Session<'a> {
    pub unsafe fn new(req: HttpRequest<'a>) -> Self {
        unsafe {
            log_to_console("[+] Starting WinHTTP request...\n");

            let user_agent = b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36\0";

            log_to_console("[*] Creating session...\n");
            let session = WinHttpOpen(
                user_agent.as_ptr() as *const u16,
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                core::ptr::null(),
                core::ptr::null(),
                0,
            );
            log_to_console("[+] Session created\n");
            log_to_console("[*] Connecting to host...\n");
            let connection_handle = WinHttpConnect(
                session,
                req.hostname.as_ptr() as *const u16,
                INTERNET_DEFAULT_HTTPS_PORT,
                0,
            );
            log_to_console("[+] Connected\n");

            log_to_console("[*] Opening request...\n");
            let request_handle = WinHttpOpenRequest(
                connection_handle,
                req.method.as_ptr() as *const u16,
                req.path.as_ptr() as *const u16,
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
                WINHTTP_FLAG_SECURE,
            );
            log_to_console("[+] Request handle created\n");

            log_to_console("[*] Adding headers...\n");
            for header in req.headers {
                if header.is_empty() || header[0] == 0 {
                    continue;
                }
                WinHttpAddRequestHeaders(
                    request_handle,
                    header.as_ptr() as *const u16,
                    u32::MAX,
                    WINHTTP_ADDREQ_FLAG_ADD,
                );
                log_to_console("[+] Header added\n");
            }

            Session {
                session,
                connection_handle,
                request_handle,
                http_request: req,
            }
        }
    }

    pub fn read_response(&self, buffer: &mut [u8]) -> Result<(), u32> {
        unsafe {
            log_to_console("[*] Receiving response...\n");
            WinHttpReceiveResponse(self.request_handle, core::ptr::null_mut());

            log_to_console("[+] Response received\n");

            log_to_console("[*] Reading response body...\n");
            let mut bytes_read = 0u32;

            loop {
                let success = WinHttpReadData(
                    self.request_handle,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as u32,
                    &mut bytes_read,
                );
                if success == 0 {
                    return Err(GetLastError());
                }
                if bytes_read == 0 {
                    break;
                }
            }
            log_to_console("[+] Wrote response to buffer\n");
            Ok(())
        }
    }

    pub fn send_request(&self) -> Result<(), u32> {
        log_to_console("[*] Preparing body...\n");
        let body_ptr = self
            .http_request
            .body
            .map_or(core::ptr::null(), |b| b.as_ptr() as *const _);
        let body_len = self.http_request.body.map_or(0, |b| b.len() as u32);
        log_to_console("[+] Body prepared\n");

        log_to_console("[*] Sending request...\n");
        unsafe {
            let result = WinHttpSendRequest(
                self.request_handle,
                core::ptr::null(),
                0,
                body_ptr,
                body_len,
                body_len,
                0,
            );

            if result == 1 {
                log_to_console("[+] Request sent\n");
                Ok(())
            } else {
                Err(GetLastError())
            }
        }
    }
}
