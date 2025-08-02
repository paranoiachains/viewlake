#![no_std]

use core::mem::zeroed;
use core::panic::PanicInfo;

use core::ffi::c_void;
use core::ptr::copy_nonoverlapping;
use windows_sys::Win32::Networking::WinSock::*;
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

pub fn resolve_domain(domain: &str) -> core::option::Option<SOCKADDR_IN> {
    unsafe {
        let mut hints: ADDRINFOA = core::mem::zeroed();
        hints.ai_family = AF_INET as i32;
        hints.ai_protocol = IPPROTO_TCP as i32;

        let mut result_ptr: *mut ADDRINFOA = core::ptr::null_mut();
        let domain_bytes = domain.as_bytes();
        let mut buffer = [0u8; 256];
        let len = domain_bytes.len();

        if len + 1 > buffer.len() {
            return core::option::Option::None;
        }

        buffer[..len].copy_from_slice(domain_bytes);
        buffer[len] = 0;

        let ret = getaddrinfo(
            buffer.as_ptr() as *const u8,
            core::ptr::null(),
            &hints,
            &mut result_ptr,
        );
        if ret != 0 || result_ptr.is_null() {
            log_to_console("getaddrinfo failed\n");
            return core::option::Option::None;
        }

        let sockaddr_in = {
            let ai = &*result_ptr;
            *(ai.ai_addr as *const SOCKADDR_IN)
        };

        freeaddrinfo(result_ptr);
        core::option::Option::Some(sockaddr_in)
    }
}

pub fn send_http_request(domain: &str, path: &str) -> (i32, [u8; 512]) {
    unsafe {
        let mut wsa_data: WSADATA = zeroed();
        if WSAStartup((2u16) | (2u16 << 8), &mut wsa_data) != 0 {
            log_to_console("WSAStartup failed\n");
            ExitProcess(1);
        }
        let addr = match resolve_domain(domain) {
            core::option::Option::Some(mut a) => {
                a.sin_port = 80u16.to_be();
                a
            }
            core::option::Option::None => {
                WSACleanup();
                ExitProcess(1);
            }
        };

        let sock = socket(AF_INET as i32, SOCK_STREAM as i32, IPPROTO_TCP as i32);
        if sock == INVALID_SOCKET {
            log_to_console("socket creation failed\n");
            WSACleanup();
            ExitProcess(1);
        }

        let ret = connect(
            sock,
            &addr as *const _ as *const SOCKADDR,
            core::mem::size_of::<SOCKADDR_IN>() as i32,
        );
        if ret == SOCKET_ERROR {
            log_to_console("connect failed\n");
            closesocket(sock);
            WSACleanup();
            ExitProcess(1);
        }

        let mut req_buf = [0u8; 256];
        let request = alloc_http_request_no_alloc(domain, path, &mut req_buf);
        let send_res = send(sock, request.as_ptr() as *const u8, request.len() as i32, 0);

        if send_res == SOCKET_ERROR {
            log_to_console("send failed\n");
            closesocket(sock);
            WSACleanup();
            ExitProcess(1);
        }

        let mut buffer = [0u8; 512];
        let received = recv(sock, buffer.as_mut_ptr() as *mut u8, buffer.len() as i32, 0);

        if received == SOCKET_ERROR {
            log_to_console("recv failed\n");
        } else {
            log_to_console("recv succedeed\n");
        }

        closesocket(sock);
        WSACleanup();

        (received, buffer)
    }
}

pub fn log_bytes_to_console(bytes: &[u8]) {
    unsafe {
        let console = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut written = 0u32;
        let _ = WriteConsoleA(
            console,
            bytes.as_ptr() as *const c_void,
            bytes.len() as u32,
            &mut written as *mut u32,
            core::ptr::null_mut(),
        );
    }
}

pub fn alloc_http_request_no_alloc<'a>(domain: &str, path: &str, buffer: &'a mut [u8]) -> &'a [u8] {
    let mut offset = 0;

    fn write_str(dst: &mut [u8], offset: &mut usize, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), dst[*offset..].as_mut_ptr(), len);
        }
        *offset += len;
    }

    write_str(buffer, &mut offset, "GET ");
    write_str(buffer, &mut offset, path);
    write_str(buffer, &mut offset, " HTTP/1.1\r\nHost: ");
    write_str(buffer, &mut offset, domain);
    write_str(buffer, &mut offset, "\r\nConnection: close\r\n\r\n");

    &buffer[..offset]
}
