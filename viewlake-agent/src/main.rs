#![no_main]

use log::info;
use viewlake_agent::run;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    #[cfg(feature = "logging")]
    viewlake_agent::init_logging();

    info!("Started logger");

    let _ = run().unwrap();
    0
}
