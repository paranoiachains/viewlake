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

        let raw_response = self.client.receive(&handle)?;

        Ok(Response::new(raw_response))
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
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    fn new(raw: String) -> Self {
        let mut lines = raw.lines();

        let status_line = lines.next().unwrap_or("");
        let mut parts = status_line.split_whitespace();
        let code = parts
            .next()
            .and_then(|c| c.parse::<u16>().ok())
            .unwrap_or(0);

        let mut headers = HashMap::new();
        for line in &mut lines {
            if line.trim().is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        let body_str = lines.collect::<Vec<_>>().join("\n");
        let body = body_str.into_bytes();

        Self {
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

        let response = comm.request(request).expect("Request failed");

        println!("Status code: {}", response.code);
        println!("Headers: {:?}", response.headers);
        println!(
            "Body (first 200 chars): {}",
            String::from_utf8_lossy(&response.body[..std::cmp::min(200, response.body.len())])
        );

        assert!(
            response.code >= 200 && response.code < 300,
            "Expected 2xx status code"
        );
        assert!(!response.body.is_empty(), "Body should not be empty");
        assert!(
            response.headers.contains_key("Content-Type")
                || response.headers.contains_key("content-type")
        );
    }

    #[test]
    fn get_request_with_headers() {
        let mut comm = Communicator::new().unwrap();

        let request = Request::new(
            TEST_HOST,
            TEST_PORT,
            "GET",
            TEST_PATH,
            Some(vec!["User-Agent: RustTestClient"]),
            None,
        );

        let response = comm.request(request).unwrap();

        println!("Status code: {}", response.code);
        assert!(response.code >= 200 && response.code < 300);
        assert!(!response.body.is_empty());
    }

    #[test]
    fn post_request_with_body() {
        let mut comm = Communicator::new().unwrap();

        let body_content = "Hello World";

        let request = Request::new(
            TEST_HOST,
            TEST_PORT,
            "POST",
            TEST_PATH,
            None,
            Some(body_content),
        );

        let response = comm.request(request).unwrap();

        println!("Status code: {}", response.code);
        println!(
            "Body (first 200 chars): {}",
            String::from_utf8_lossy(&response.body[..std::cmp::min(200, response.body.len())])
        );

        assert!(response.code >= 200 && response.code < 300);
        assert!(!response.body.is_empty());
    }
}
