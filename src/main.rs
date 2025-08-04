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

        if let Err(err) = session.send_request() {
            log_to_console("[-] Failed to send request\n");
            log_error_code(err);
            ExitProcess(1);
        }

        let mut response_buffer = [0u8; 1024];
        if let Err(err) = session.read_response(&mut response_buffer) {
            log_to_console("[-] Failed to read response\n");
            log_error_code(err);
            ExitProcess(1);
        }

        log_to_console("[+] Request and response completed\n");
        ExitProcess(0);
    }
}

pub fn log_error_code(code: u32) {
    use core::fmt::Write;
    let mut buf = [0u8; 16];
    let len = utoa(code, &mut buf);
    unsafe {
        log_to_console("Error code: ");
        log_to_console(core::str::from_utf8_unchecked(&buf[..len]));
        log_to_console("\n");
    }
}

fn utoa(mut num: u32, out: &mut [u8]) -> usize {
    if num == 0 {
        out[0] = b'0';
        return 1;
    }
    let mut i = out.len();
    while num > 0 {
        i -= 1;
        out[i] = b'0' + (num % 10) as u8;
        num /= 10;
    }
    let len = out.len() - i;
    out.copy_within(i.., 0);
    len
}
