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

        buffers.hostname_len = ascii_to_utf16(b"example.com", &mut buffers.hostname);
        buffers.path_len = ascii_to_utf16(b"/", &mut buffers.path);
        buffers.method_len = ascii_to_utf16(b"GET", &mut buffers.method);
        buffers.headers_count = 1;
        buffers.headers_len[0] = ascii_to_utf16(b"Accept: */*", &mut buffers.headers[0]);
        buffers.accept_len = ascii_to_utf16(b"text/plain", &mut buffers.accept);

        let mut header_refs: [&[u16]; 16] = [&[]; 16];
        let request = HttpRequest::new(&buffers, &mut header_refs);
        let session = Session::new(request);
        if let Err(_) = session.send_request() {
            log_to_console("[-] Failed to send request\n");
            ExitProcess(1);
        }

        let mut response_buffer = [0u8; 1024];
        if let Err(_) = session.read_response(&mut response_buffer) {
            log_to_console("[-] Failed to read response\n");
            ExitProcess(1);
        }

        log_to_console("[+] Request and response completed\n");

        ExitProcess(0);
    }
}
