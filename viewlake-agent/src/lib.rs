mod beacon;
mod collector;

#[cfg(feature = "logging")]
pub fn init_logging() {
    simple_logger::init().unwrap();
}

pub fn run() -> windows::core::Result<()> {
    let mut beacon = beacon::Beacon::new()?;
    beacon.init_conn("example.com", 443)?;

    Ok(())
}
