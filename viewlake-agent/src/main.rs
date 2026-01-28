#![no_main]

use log::{error, info};
use viewlake_agent::run;
use windows::core::HSTRING;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    #[cfg(feature = "logging")]
    viewlake_agent::init_logging();

    info!("started logger");

    let hostname = HSTRING::from("127.0.0.1");
    let port = 9090;

    info!("home's addr: {}:{}", hostname, port);

    if let Err(e) = run(hostname, port) {
        error!("got error: {}", e);
    }

    0
}
