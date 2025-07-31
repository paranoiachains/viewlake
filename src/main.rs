#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use windows_sys::Win32::Networking::WinSock::SOCKET_ERROR;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        viewlake::log_to_console("starting http request...\n");
        let received = viewlake::send_http_request("ya.ru", "/");
        if received == SOCKET_ERROR {
            viewlake::log_to_console("Request failed\n");
        } else {
            viewlake::log_to_console("Request succeeded\n");
        }

        ExitProcess(0);
    }
}
