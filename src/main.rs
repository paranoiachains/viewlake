#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use viewlake::panic;
use windows_sys::Win32::System::Threading::ExitProcess;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn mainCRTStartup() -> ! {
    unsafe {
        ExitProcess(0);
    }
}
