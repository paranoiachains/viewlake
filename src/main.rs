#![no_main]
#![no_std]
#![windows_subsystem = "console"]

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
