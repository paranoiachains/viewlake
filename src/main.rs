#![no_main]

use viewlake::client::{Client, Request};

#[unsafe(no_mangle)]
pub fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    1
}
