#![no_main]

use viewlake::collect;
use viewlake::send_initial;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    collect().unwrap();
    let response = send_initial().unwrap();
    println!("{}, {:?}", response.code, response.body);

    0
}
