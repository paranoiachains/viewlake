#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use viewlake::*;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        let mut buffers = Utf16Buffers {
            hostname: [0; 256],
            hostname_len: 0,
            path: [0; 256],
            path_len: 0,
            method: [0; 16],
            method_len: 0,
            headers: [[0; 256]; 16],
            headers_len: [0; 16],
            headers_count: 0,
            accept: [0; 64],
            accept_len: 0,
            body: Some((b"hello", 5)),
        };
        ExitProcess(0);
    }
}
