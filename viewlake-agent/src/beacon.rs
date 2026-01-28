#![allow(dead_code)]
pub mod client;
use client::*;
use log::info;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::HSTRING;

pub struct Beacon {
    pub id: u32,
    client: Client,
}

impl Beacon {
    pub fn new() -> windows::core::Result<Self> {
        let client = Client::new()?;
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        Ok(Self { client, id })
    }

    pub fn init_conn(&mut self, hostname: HSTRING, port: u16) -> windows::core::Result<()> {
        info!("sending initial request to {hostname}:{port}");
        let id_bytes = self.id.to_le_bytes();

        let req = Request {
            hostname,
            port,
            method: HSTRING::from("POST"),
            path: HSTRING::from("/hello"),
            headers: None,
            body: Some(&id_bytes),
        };

        let resp = self.client.request(req)?;

        info!("response body: {}", resp.body);
        info!("response headers: {}", resp.headers);

        Ok(())
    }
}
