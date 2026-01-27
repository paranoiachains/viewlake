#![no_main]

use log::{error, info};
use viewlake_agent::run;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    #[cfg(feature = "logging")]
    viewlake_agent::init_logging();

    info!("Started logger");

    if let Err(e) = run() {
        error!("got error: {}", e);
    }
    0
}
