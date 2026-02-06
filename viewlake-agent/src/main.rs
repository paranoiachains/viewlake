#![no_main]

use std::net::SocketAddrV4;

use log::{error, info};
use viewlake_agent::run;

#[cfg(feature = "logging")]
pub fn init_logging() {
    simple_logger::init().unwrap();
}

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    #[cfg(feature = "logging")]
    init_logging();

    info!("logging feature enabled");

    let home = SocketAddrV4::from("127.0.0.1:3000".parse().unwrap());

    info!("home's addr: {}:{}", home.ip(), home.port());

    if let Err(e) = run(home) {
        error!("got error: {}", e);
    }

    0
}
