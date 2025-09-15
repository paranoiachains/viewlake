mod client;
use crate::comms::client::*;
use windows::core::Error;

pub struct Beacon {
    client: Client,
}

impl Beacon {
    pub fn new() -> Result<Self, Error> {
        let client = Client::new()?;
        Ok(Self { client })
    }
}
