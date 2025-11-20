mod beacon;
mod collector;

#[cfg(feature = "logging")]
pub fn init_logging() {
    simple_logger::init().unwrap();
}
