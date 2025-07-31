#![no_std]
#![no_main]

use viewlake::*;
use windows_sys::Win32::Networking::WinSock::SOCKET_ERROR;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        log_to_console("starting http request...\n");
        let received = send_http_request("example.com", "/");
        if received == SOCKET_ERROR {
            log_to_console("Request failed\n");
        } else {
            log_to_console("Request succeeded\n");
        }

        ExitProcess(0);
    }
}
