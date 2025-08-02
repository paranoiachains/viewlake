#![no_std]
#![allow(unused_imports)]

use core::any::{Any, TypeId};
use core::ffi::c_void;
use core::panic::PanicInfo;
use core::ptr::null;

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

#[cfg(not(feature = "logging"))]
pub fn log_to_console(_s: &str) {}

pub struct HttpRequest<'a> {
    pub hostname: &'a [u16],
    pub path: &'a [u16],
    pub method: &'a [u16],
    pub headers: [&'a [u16]; 16],
    pub body: Option<&'a [u8]>,
    pub accept: &'a [u16],
}

impl<'a> HttpRequest<'a> {
    pub fn new(
        hostname: &[u8],
        path: &[u8],
        method: &[u8],
        headers_utf8: [&[u8]; 16],
        body: Option<&'a [u8]>,
        accept: &[u8],
        hostname_buf: &'a mut [u16],
        path_buf: &'a mut [u16],
        method_buf: &'a mut [u16],
        headers_bufs: &'a mut [[u16; 256]; 16],
        accept_buf: &'a mut [u16],
    ) -> Result<Self, ()> {
        let hostname = utf8_bytes_to_utf16(hostname, hostname_buf)?;
        let path = utf8_bytes_to_utf16(path, path_buf)?;
        let method = utf8_bytes_to_utf16(method, method_buf)?;
        let accept = utf8_bytes_to_utf16(accept, accept_buf)?;

        let mut headers: [&[u16]; 16] = [&[]; 16];

        for (i, (header_utf8, buf)) in headers_utf8.iter().zip(headers_bufs.iter_mut()).enumerate()
        {
            if header_utf8.is_empty() {
                continue;
            }
            headers[i] = utf8_bytes_to_utf16(header_utf8, buf)?;
        }

        Ok(HttpRequest {
            hostname,
            path,
            method,
            headers,
            body,
            accept,
        })
    }
}

fn utf8_bytes_to_utf16<'a>(src: &[u8], dst: &'a mut [u16]) -> Result<&'a [u16], ()> {
    let s = core::str::from_utf8(src).map_err(|_| ())?;

    let mut i = 0;
    for c in s.encode_utf16() {
        if i >= dst.len() {
            return Err(());
        }
        dst[i] = c;
        i += 1;
    }

    if i >= dst.len() {
        return Err(());
    }
    dst[i] = 0;

    Ok(&dst[..i + 1])
}

pub unsafe fn request(req: &HttpRequest) {
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
                u32::MAX, // full header string
                WINHTTP_ADDREQ_FLAG_ADD,
            );
            log_to_console("[+] Header added\n");
        }

        log_to_console("[*] Preparing body...\n");
        let body_ptr = req
            .body
            .map_or(core::ptr::null(), |b| b.as_ptr() as *const _);
        let body_len = req.body.map_or(0, |b| b.len() as u32);
        log_to_console("[+] Body prepared\n");

        log_to_console("[*] Sending request...\n");
        WinHttpSendRequest(
            request_handle,
            core::ptr::null(),
            0,
            body_ptr,
            body_len,
            body_len,
            0,
        );
        log_to_console("[+] Request sent\n");

        log_to_console("[*] Receiving response...\n");
        WinHttpReceiveResponse(request_handle, core::ptr::null_mut());
        log_to_console("[+] Response received\n");

        log_to_console("[*] Reading response body...\n");
        let mut buffer = [0u8; 4096];
        let mut bytes_read = 0u32;

        loop {
            let success = WinHttpReadData(
                request_handle,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
            );
            if success == 0 || bytes_read == 0 {
                break;
            }
            log_to_console(
                core::str::from_utf8(&buffer[..bytes_read as usize]).unwrap_or("[invalid utf8]\n"),
            );
        }

        log_to_console("[+] Finished reading response\n");

        log_to_console("[*] Cleaning up...\n");
        WinHttpCloseHandle(request_handle);
        WinHttpCloseHandle(connection_handle);
        WinHttpCloseHandle(session);
        log_to_console("[+] Done\n");
    }
}
