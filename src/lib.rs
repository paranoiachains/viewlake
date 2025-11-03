mod collector;
mod comms;

use collector::SystemFingerprint;
use comms::{Request, Response};
use windows::core::Result;

use crate::comms::Communicator;

pub fn collect() -> Result<SystemFingerprint> {
    SystemFingerprint::collect()
}

pub fn send_initial() -> Result<Response> {
    let mut comm = Communicator::new()?;
    let request = Request::new("www.example.com", 443, "GET", "/", None, None);

    comm.request(request)
}
