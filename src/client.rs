use crate::winhttp::{WinHttpConnection, WinHttpRequest, WinHttpSession};
use std::io::Write;
use windows::core::{Error, HSTRING};

pub struct Client {
    pub session: WinHttpSession,
    pub connection: Option<WinHttpConnection>, // Store hostname with connection
}

const DEFAULT_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

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
        if self
            .connection
            .as_ref()
            .map_or(true, |c| c.hostname != request.hostname)
        {
            self.connection = Some(WinHttpConnection::new(
                &self.session,
                &request.hostname,
                request.port,
            )?);
        }
        let connection = self.connection.as_ref().unwrap();
        let win_request = WinHttpRequest::new(&connection, request.method, request.path)?;

        win_request.send(request.headers, request.body)?;
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
    pub hostname: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,
    pub headers: Option<Vec<&'a str>>,
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

    fn build_request<'a>(
        hostname: &'a str,
        port: u16,
        method: &'a str,
        path: &'a str,
        headers: Option<Vec<&'a str>>,
        body: Option<&'a str>,
    ) -> Request<'a> {
        Request {
            hostname,
            port,
            method,
            path,
            headers,
            body,
        }
    }

    #[test]
    fn create_client() {
        Client::new().expect("Failed to create client");
    }

    #[test]
    fn send_request_with_body_and_headers() {
        let mut client = Client::new().expect("Failed to create client");

        let request = build_request(
            "httpbin.org",
            443,
            "GET",
            "/anything",
            Some(vec!["Header1: 1", "Header2: 2"]),
            Some("Body"),
        );

        let handle = client
            .send_request(request)
            .expect("Failed to send request");
        let response = client
            .receive_response(&handle.0)
            .expect("Failed to receive response");

        response.read_response_into_stdout();
    }

    #[test]
    fn send_request_without_body_and_headers() {
        let mut client = Client::new().expect("Failed to create client");

        let request = build_request("httpbin.org", 443, "GET", "/anything", None, None);

        let handle = client
            .send_request(request)
            .expect("Failed to send request");
        let _response = client
            .receive_response(&handle.0)
            .expect("Failed to receive response");
    }

    #[test]
    fn reuse_connection_for_same_host() {
        let mut client = Client::new().expect("Failed to create client");

        let request_1 = build_request("httpbin.org", 443, "GET", "/anything", None, None);
        let handle_1 = client
            .send_request(request_1)
            .expect("Failed to send request");
        let _response_1 = client
            .receive_response(&handle_1.0)
            .expect("Failed to receive response");

        let request_2 = build_request("httpbin.org", 443, "POST", "/anything", None, None);
        let handle_2 = client
            .send_request(request_2)
            .expect("Failed to send request");
        let _response_2 = client
            .receive_response(&handle_2.0)
            .expect("Failed to receive response");
    }

    #[test]
    fn new_connection_for_different_host() {
        let mut client = Client::new().expect("Failed to create client");

        let request_1 = build_request("httpbin.org", 443, "GET", "/anything", None, None);
        let handle_1 = client
            .send_request(request_1)
            .expect("Failed to send request");
        let _response_1 = client
            .receive_response(&handle_1.0)
            .expect("Failed to receive response");

        let request_2 = build_request("example.com", 443, "GET", "/", None, None);
        let handle_2 = client
            .send_request(request_2)
            .expect("Failed to send request");
        let _response_2 = client
            .receive_response(&handle_2.0)
            .expect("Failed to receive response");
    }
}
