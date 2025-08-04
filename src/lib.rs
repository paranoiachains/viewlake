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

pub fn log_err(e: u32) {
    let mut buf = [0u8; 11];
    let mut i = buf.len();

    let mut n = e;
    if n == 0 {
        i -= 1;
        buf[i] = b'0';
    } else {
        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }

    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        let _ = WriteConsoleA(
            console,
            buf[i..].as_ptr() as *const c_void,
            (buf.len() - i) as u32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        let _ = WriteConsoleA(
            console,
            b"\n".as_ptr() as *const c_void,
            1,
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
    pub fn new(buffers: &'a Utf16Buffers, header_refs: &'a mut [&'a [u16]]) -> Self {
        for i in 0..buffers.headers_count {
            header_refs[i] = &buffers.headers[i][..buffers.headers_len[i]];
        }

        HttpRequest {
            hostname: &buffers.hostname[..buffers.hostname_len],
            path: &buffers.path[..buffers.path_len],
            method: &buffers.method[..buffers.method_len],
            headers: &header_refs[..buffers.headers_count],
            body: buffers
                .body
                .map(|(arr, len)| &arr[..len])
                .map(|s| unsafe { core::slice::from_raw_parts(s.as_ptr() as *const u8, s.len()) }),
            accept: &buffers.accept[..buffers.accept_len],
        }
    }
}

pub fn ascii_to_utf16(input: &[u8], out: &mut [u16]) -> usize {
    let mut i = 0;
    while i < input.len() && i < out.len() {
        out[i] = input[i] as u16;
        i += 1;
    }
    i
}

pub struct Utf16Buffers {
    pub hostname: [u16; 256],
    pub hostname_len: usize,
    pub path: [u16; 256],
    pub path_len: usize,
    pub method: [u16; 16],
    pub method_len: usize,
    pub headers: [[u16; 256]; 16],
    pub headers_len: [usize; 16],
    pub headers_count: usize,
    pub accept: [u16; 64],
    pub accept_len: usize,
    pub body: Option<(&'static [u8], usize)>,
}

pub struct Session<'a> {
    pub session: *mut c_void,
    pub connection_handle: *mut c_void,
    pub request_handle: *mut c_void,
    pub http_request: HttpRequest<'a>,
}

impl<'a> Session<'a> {
    pub fn new(req: HttpRequest<'_>) -> Session {
        unsafe {
            log_to_console("[+] Starting WinHTTP request...\n");

            let user_agent = b"Mozilla/5.0\0";

            log_to_console("[*] Creating session...\n");
            let session = WinHttpOpen(
                user_agent.as_ptr() as *const u16,
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                core::ptr::null(),
                core::ptr::null(),
                0,
            );

            if session.is_null() {
                log_to_console("[-] WinHttpOpen failed\n");
                log_err(GetLastError());
                ExitProcess(1);
            }

            log_to_console("[*] Connecting to host...\n");
            let connection = WinHttpConnect(
                session,
                req.hostname.as_ptr(),
                INTERNET_DEFAULT_HTTPS_PORT,
                0,
            );

            if connection.is_null() {
                log_to_console("[-] WinHttpConnect failed\n");
                log_err(GetLastError());
                ExitProcess(1);
            }

            log_to_console("[*] Opening request...\n");
            let request = WinHttpOpenRequest(
                connection,
                req.method.as_ptr(),
                req.path.as_ptr(),
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
                WINHTTP_FLAG_SECURE,
            );

            if request.is_null() {
                log_to_console("[-] WinHttpOpenRequest failed\n");
                log_err(GetLastError());
                ExitProcess(1);
            }

            log_to_console("[+] HTTP session initialized\n");

            Session {
                session,
                connection_handle: connection,
                request_handle: request,
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
