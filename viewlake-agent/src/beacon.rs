pub mod client;
use client::*;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::Result;

pub struct Beacon {
    pub id: u32,
    client: Client,
    home_ip: Box<str>,
    home_port: u16,
}

impl Beacon {
    pub fn new(home_ip: &str, home_port: u16) -> Result<Self> {
        let client = Client::new()?;
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        Ok(Self {
            client,
            home_ip: home_ip.into(),
            home_port,
            id,
        })
    }
}
