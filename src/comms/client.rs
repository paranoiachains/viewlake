#![allow(dead_code)]
/// High level API for WinHTTP
mod winhttp;

use crate::comms::client::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use std::os::raw::c_void;
use windows::core::Result;

/// Abstraction over WinHttpSession and WinHttpConnection
pub struct Client {
    session: WinHttpSession,
    connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: &'static str = "SomeAgent"; // TODO: randomize user-agent

pub struct RequestHandle(WinHttpRequest);

impl RequestHandle {
    pub fn read(&self) -> Result<Vec<u8>> {
        let mut body = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            let bytes_read = self
                .0
                .read(buf.as_mut_ptr() as *mut c_void, buf.len() as u32)?;

            if bytes_read == 0 {
                break; // no more data
            }

            body.extend_from_slice(&buf[..bytes_read as usize]);
        }

        Ok(body)
    }

    fn receive(&self) -> Result<()> {
        self.0.receive()
    }
}

impl Client {
    pub fn new() -> Result<Self> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
        })
    }

    /// Sends request and returns RequestHandle, which is supposed to be passed to receive_response
    pub fn send(
        &mut self,
        hostname: &str,
        port: u16,
        method: &str,
        path: &str,
        headers: Option<Vec<&str>>,
        body: Option<&str>,
    ) -> Result<RequestHandle> {
        if self
            .connection
            .as_ref()
            .map_or(true, |c| c.hostname != hostname)
        {
            self.connection = Some(WinHttpConnection::new(&self.session, &hostname, port)?);
        }
        let connection = self.connection.as_ref().unwrap();
        let win_request = WinHttpRequest::new(&connection, method, path)?;

        win_request.send(headers, body)?;
        Ok(RequestHandle(win_request))
    }

    /// Receives response using provided RequestHandle obtained from send_request func
    pub fn receive(&self, handle: &RequestHandle) -> Result<String> {
        handle.receive()?;

        let response = handle.read()?;
        println!("Response: {:?}", response);

        Ok(String::from_utf8_lossy(&response).into_owned())
    }
}
