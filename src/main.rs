#![no_main]
#![windows_subsystem = "console"]

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        ExitProcess(0);
    }
}
