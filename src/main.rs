#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use viewlake::panic;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        viewlake::log_to_console("hello\n");
        let mut hostname_buf = [0u16; 256];
        let mut path_buf = [0u16; 256];
        let mut method_buf = [0u16; 32];
        let mut accept_buf = [0u16; 64];
        let mut headers_bufs = [[0u16; 256]; 16];

        let req = viewlake::HttpRequest::new(
            b"ya.ru",
            b"/",
            b"GET",
            [
                b"User-Agent: Test\0",
                b"Accept: */*\0",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
                b"",
            ],
            None,
            b"*/*",
            &mut hostname_buf,
            &mut path_buf,
            &mut method_buf,
            &mut headers_bufs,
            &mut accept_buf,
        )
        .unwrap();
        viewlake::request(&req);
        ExitProcess(0);
    }
}
