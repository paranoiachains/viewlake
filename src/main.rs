#![no_main]
#![windows_subsystem = "console"]

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[no_mangle]
pub fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    let mut stdout = stdout();
    stdout.write_all(b"Hello, world!\n").unwrap();

    0
}
