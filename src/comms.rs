mod client;
use crate::comms::client::*;
use std::collections::HashMap;
use windows::core::Result;

pub struct Communicator {
    client: Client,
}

impl Communicator {
    pub fn new() -> Result<Self> {
        let client = Client::new()?;
        Ok(Self { client })
    }

    /// Send request and receive response
    pub fn request(&mut self, request: Request) -> Result<Response> {
        let handle = self.client.send(
            request.hostname,
            request.port,
            request.method,
            request.path,
            request.headers,
            request.body,
        )?;

        self.client.receive(&handle)?;

        let code = handle.0.status_code()?;
        let headers = handle.0.headers()?;
        let body = handle.read()?;

        Ok(Response::new(code, headers, body))
    }
}

pub struct Request<'a> {
    hostname: &'a str,
    port: u16,
    method: &'a str,
    path: &'a str,
    headers: Option<Vec<&'a str>>,
    body: Option<&'a str>,
}

impl<'a> Request<'a> {
    pub fn new(
        hostname: &'a str,
        port: u16,
        method: &'a str,
        path: &'a str,
        headers: Option<Vec<&'a str>>,
        body: Option<&'a str>,
    ) -> Self {
        Request {
            hostname,
            port,
            method,
            path,
            headers,
            body,
        }
    }
}

pub struct Response {
    pub code: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    fn new(code: u32, headers: HashMap<String, String>, body: Option<Vec<u8>>) -> Self {
        Response {
            code,
            headers,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HOST: &str = "www.example.com";
    const TEST_PORT: u16 = 443;
    const TEST_PATH: &str = "/";

    #[test]
    fn communicator_creation() {
        assert!(Communicator::new().is_ok())
    }

    #[test]
    fn get_request_basic() {
        let mut comm = Communicator::new().unwrap();
        let request = Request::new(TEST_HOST, TEST_PORT, "GET", TEST_PATH, None, None);
        let response = comm.request(request).unwrap();

        println!("Status code: {}", response.code);
        if let Some(body) = &response.body {
            println!(
                "Body (first 200 chars): {}",
                String::from_utf8_lossy(&body[..std::cmp::min(200, body.len())])
            );
        } else {
            println!("Body is empty");
        }

        assert!(
            response.code >= 200 && response.code < 300,
            "Expected 2xx status code"
        );
    }
}
