#![no_main]

use viewlake_agent::hello;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    let _ = hello().unwrap();
    0
}
