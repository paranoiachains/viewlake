mod client;
use windows::core::Error;

use client;

pub struct Beacon {
    client: Client,
}

impl Beacon {
    pub fn new() -> Result<Self, Error> {
        let client = client::Client::new()?;
        Ok(client)
    }
}
