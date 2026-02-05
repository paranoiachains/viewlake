mod beacon;
pub mod collector;

use std::net::SocketAddrV4;

#[cfg(feature = "logging")]
pub fn init_logging() {
    simple_logger::init().unwrap();
}

pub fn run(home: SocketAddrV4) -> windows::core::Result<()> {
    let mut beacon = beacon::Beacon::new(home)?;
    beacon.initial_request()?;

    beacon.send_system_info()?;

    Ok(())
}
