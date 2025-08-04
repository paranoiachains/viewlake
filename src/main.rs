#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use viewlake::*;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        static HOSTNAME: &[u16] = &[
            0x0065, 0x0078, 0x0061, 0x006D, 0x0070, 0x006C, 0x0065, 0x002E, 0x0063, 0x006F, 0x006D,
        ]; // "example.com"
        static METHOD: &[u16] = &[0x0047, 0x0045, 0x0054]; // "GET"
        static PATH: &[u16] = &[0x002F]; // "/"
        static ACCEPT_HEADER: &[u16] = &[
            0x0041, 0x0063, 0x0063, 0x0065, 0x0070, 0x0074, 0x003A, 0x0020, 0x002A, 0x002F, 0x002A,
        ]; // "Accept: */*"
        static ACCEPT: &[u16] = &[
            0x0074, 0x0065, 0x0078, 0x0074, 0x002F, 0x0070, 0x006C, 0x0061, 0x0069, 0x006E,
        ];

        let http_request = HttpRequest {
            hostname: HOSTNAME,
            method: METHOD,
            path: PATH,
            headers: &[ACCEPT_HEADER],
            accept: ACCEPT,
            body: None,
        };

        let session = Session::new(http_request);

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
