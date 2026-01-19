mod beacon;
mod collector;

#[cfg(feature = "logging")]
pub fn init_logging() {
    simple_logger::init().unwrap();
}

pub fn run() -> windows::core::Result<()> {
    let _beacon = beacon::Beacon::new("example.com", 443)?;

    Ok(())
}
