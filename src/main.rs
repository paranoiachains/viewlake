#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use viewlake::*;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        let buffers = utf::to_utf16(
            "example.com",
            "/",
            "GET",
            &["Accept: */*"],
            None,
            "text/plain",
        );
        let mut header_refs: [&[u16]; utf::MAX_HEADERS] = [&[]; utf::MAX_HEADERS];

        let http_request = HttpRequest::new(&buffers, &mut header_refs);
        let _session = Session::new(http_request);
        ExitProcess(0);
    }
}
