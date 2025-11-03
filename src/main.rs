#![no_main]

use viewlake::collect;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    collect().unwrap();
    0
}
