#![no_std]

use core::any::{Any, TypeId};
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

fn request(token: &str) {
    unsafe {
        let session = WinHttpOpen(b"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36\0".as_ptr() as *const u16,
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, core::ptr::null(), core::ptr::null(), 0);

        let connection_handle = WinHttpConnect(
            session,
            b"ya.ru\0".as_ptr() as *const u16,
            INTERNET_DEFAULT_HTTPS_PORT,
            0,
        );

        let request_handle = WinHttpOpenRequest(
            connection_handle,
            b"GET\0".as_ptr() as *const u16,
            b"/\0".as_ptr() as *const u16,
            core::ptr::null(),
            core::ptr::null(),
            b"application/json".as_ptr() as *const *const u16,
            WINHTTP_FLAG_SECURE,
        );

        let token_length = token.chars().count() as u32;
        let token = token.as_bytes().as_ptr() as *const u16;

        let sent = WinHttpSendRequest(
            request_handle,
            token,
            token_length,
            core::ptr::null(),
            0,
            0,
            0,
        );

        if sent == 0 {
            WinHttpCloseHandle(request_handle);
            WinHttpCloseHandle(connection_handle);
            WinHttpCloseHandle(session);
        }

        let recv = WinHttpReceiveResponse(request_handle, core::ptr::null_mut());

        if recv == 0 {
            WinHttpCloseHandle(request_handle);
            WinHttpCloseHandle(connection_handle);
            WinHttpCloseHandle(session);
        }

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
        }

        WinHttpCloseHandle(request_handle);
        WinHttpCloseHandle(connection_handle);
        WinHttpCloseHandle(session);
    }
}
