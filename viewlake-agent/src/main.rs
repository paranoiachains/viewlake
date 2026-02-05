#![no_main]

use std::net::SocketAddrV4;

use log::{error, info};
use viewlake_agent::run;
use windows::core::HSTRING;

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    #[cfg(feature = "logging")]
    viewlake_agent::init_logging();

    info!("started logger");

    let home = SocketAddrV4::from("127.0.0.1:3000");

    info!("home's addr: {}:{}", hostname, port);

    if let Err(e) = run(home) {
        error!("got error: {}", e);
    }

    0
}
