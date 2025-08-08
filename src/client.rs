use crate::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession, concat_pcwstr};
use std::os::raw::c_void;
use windows::core::{Error, PCWSTR, w};

pub struct Client {
    pub session: WinHttpSession,
    pub connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: PCWSTR = w!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
);

impl Client {
    pub fn new() -> Result<Self, Error> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
        })
    }

    pub fn send_request(&mut self, request: Request) -> Result<(), Error> {
        let hostname = &request.hostname;

        let connection = match &self.connection {
            Some(conn) if conn.hostname == unsafe { hostname.to_string().unwrap() } => conn,
            _ => &WinHttpConnection::new(&self.session, *hostname)
                .expect("Connection creation failed."),
        };

        let win_request = WinHttpRequest::new(&connection, request.method, request.path)?;
        let headers = request.headers.map(|headers_pcwstr| {
            let headers_u16 = concat_pcwstr(headers_pcwstr);
            headers_u16
        });

        let body = request.body.map(|body_str| {
            let body_ptr = body_str.as_bytes().as_ptr() as *const c_void;
            let body_len = body_str.as_bytes().len() as u32;
            (body_ptr, body_len)
        });

        win_request.send(headers.as_ref().map(|t| t.as_slice()), body)?;

        Ok(())
    }
}

pub struct Request<'a> {
    pub hostname: PCWSTR,
    pub method: PCWSTR,
    pub path: PCWSTR,
    pub headers: Option<Vec<PCWSTR>>, // \r\n at the end of each header
    pub body: Option<&'a str>,
}
