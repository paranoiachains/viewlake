use crate::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use std::{io::Write, os::raw::c_void};
use windows::core::{Error, HSTRING, PCWSTR, w};

pub struct Client {
    pub session: WinHttpSession,
    pub connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: PCWSTR = w!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
);

pub struct RequestHandle(WinHttpRequest);

impl Client {
    pub fn new() -> Result<Self, Error> {
        let session = WinHttpSession::new(DEFAULT_AGENT)?;
        Ok(Client {
            session,
            connection: None,
        })
    }

    pub fn send_request(&mut self, request: Request) -> Result<RequestHandle, Error> {
        let hostname = &request.hostname;

        let connection = match &self.connection {
            Some(conn) if conn.hostname == unsafe { hostname.to_string().unwrap() } => conn,
            _ => &WinHttpConnection::new(&self.session, *hostname)
                .expect("Connection creation failed."),
        };

        let win_request = WinHttpRequest::new(&connection, request.method, request.path)?;

        let body = request.body.map(|body_str| {
            let body_ptr = body_str.as_bytes().as_ptr() as *const c_void;
            let body_len = body_str.as_bytes().len() as u32;
            (body_ptr, body_len)
        });

        win_request.send(request.headers, body)?;
        Ok(RequestHandle(win_request))
    }

    pub fn receive_response(&self, handle: &WinHttpRequest) -> Result<Response, Error> {
        handle.receive()?;

        let mut buf = [0u8; 4096];
        handle.read(buf.as_mut_ptr() as *mut _, buf.len() as u32)?;

        match std::str::from_utf8(&buf) {
            Ok(data) => Ok(Response {
                data: data.to_string(),
            }),
            Err(_) => Err(Error::new(windows::core::HRESULT(1), HSTRING::new())),
        }
    }
}

pub struct Request<'a> {
    pub hostname: PCWSTR,
    pub method: PCWSTR,
    pub path: PCWSTR,
    pub headers: Option<Vec<String>>, // \r\n at the end of each header
    pub body: Option<&'a str>,
}

pub struct Response {
    pub data: String,
}

impl Response {
    pub fn read_response_into_stdout(&self) {
        std::io::stdout().write_all(self.data.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client() {
        Client::new().expect("Client creation error");
    }

    #[test]
    fn send_request() {
        let mut client = Client::new().expect("Client creation error");
        let request = Request {
            hostname: w!("httpbin.org"),
            method: w!("GET"),
            path: w!("/anything"),
            headers: Some(vec![String::from("Header1: 1"), String::from("Header2: 2")]),
            body: Some("Body"),
        };
        let handle = client.send_request(request).expect("Send request error");
        let response = client
            .receive_response(&handle.0)
            .expect("Receive response error");

        response.read_response_into_stdout();
    }
}
