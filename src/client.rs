use core::ffi::c_void;
use std::ffi::OsStr;

use std::os::windows::ffi::OsStrExt;

use windows::core::PCWSTR;

use crate::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use windows::core::Error;

pub struct Client {
    pub session: WinHttpSession,
    pub connection: Option<WinHttpConnection>,
    agent_utf16: Vec<u16>, // neccessary for utf16 conversions
    host_utf16: Option<Vec<u16>>,
}

fn utf8_to_utf16(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

impl Client {
    /// To establish connection immediately, provide hostname. Else, it will be established during
    /// send_request method.
    pub fn new(agent: &str, hostname: Option<&str>) -> Result<Self, Error> {
        let agent_utf16 = utf8_to_utf16(agent);
        let agent_wide = PCWSTR::from_raw(agent_utf16.as_ptr());

        let session = WinHttpSession::new(agent_wide)?;

        let (connection, host_utf16) = if let Some(host) = hostname {
            let host_utf16 = utf8_to_utf16(host);
            let host_wide = PCWSTR::from_raw(host_utf16.as_ptr());
            let conn = WinHttpConnection::new(&session, host_wide)?;
            (Some(conn), Some(host_utf16))
        } else {
            (None, None)
        };

        Ok(Client {
            session,
            connection,
            agent_utf16,
            host_utf16,
        })
    }

    pub fn send_request(&mut self, request: Request) -> Result<(), Error> {
        let host_utf16 = utf8_to_utf16(request.hostname);
        let host_wide = PCWSTR::from_raw(host_utf16.as_ptr());

        match &self.connection {
            Some(conn) => {
                if conn.hostname != request.hostname {
                    self.connection = Some(WinHttpConnection::new(&self.session, host_wide)?);
                }
            }
            None => {
                self.connection = Some(WinHttpConnection::new(&self.session, host_wide)?);
            }
        }

        let method_utf16 = utf8_to_utf16(request.method);
        let method_wide = PCWSTR::from_raw(method_utf16.as_ptr());
        let path_utf16 = utf8_to_utf16(request.path);
        let path_wide = PCWSTR::from_raw(path_utf16.as_ptr());
        let accept_type_utf16 = utf8_to_utf16(request.accept_type);
        let accept_type_wide = PCWSTR::from_raw(accept_type_utf16.as_ptr());

        let request_handle = WinHttpRequest::new(
            self.connection.as_ref().unwrap(),
            method_wide,
            path_wide,
            &accept_type_wide,
        )?;

        let headers_joined = request.headers.map(|h| {
            let joined = h.join("\r\n");
            utf8_to_utf16(joined.as_str())
        });

        let headers_wide: Option<&[u16]> = headers_joined.as_ref().map(|v| v.as_slice());

        let body_wide = request.body.map(|b| {
            let bytes = b.as_bytes();
            (bytes.as_ptr() as *const c_void, bytes.len() as u32)
        });

        request_handle.send(headers_wide, body_wide)?;
        request_handle.receive()?;

        const BUF_LEN: u32 = 256;

        let buf = [0u16; BUF_LEN as usize].as_ptr() as *mut c_void;

        if let Err(e) = request_handle.read_response(buf, BUF_LEN) {
            return Err(e);
        };

        Ok(())
    }
}

pub struct Request<'a> {
    pub hostname: &'a str,
    pub method: &'a str,
    pub path: &'a str,
    pub accept_type: &'a str,
    pub headers: Option<Vec<&'a str>>,
    pub body: Option<&'a str>,
}
