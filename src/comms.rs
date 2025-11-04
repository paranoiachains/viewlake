pub mod client;
use crate::comms::client::*;
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
        self.client.send_and_read(
            request.hostname,
            request.port,
            request.method,
            request.path,
            request.headers,
            request.body,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const TEST_HOST: &str = "www.example.com";
    const TEST_PORT: u16 = 443;
    const TEST_PATH: &str = "/";

    #[test]
    fn communicator_creation() {
        assert!(
            Communicator::new().is_ok(),
            "Communicator creation should not return Err"
        )
    }

    #[test]
    fn get_request_basic() {
        let mut comm = Communicator::new().unwrap();
        let request = Request::new(TEST_HOST, TEST_PORT, "GET", TEST_PATH, None, None);
        let response = comm
            .request(request)
            .expect("Sending request threw an error");

        println!("Status code: {}", response.code);
        if let Some(body) = &response.body {
            println!("{}", body);
        } else {
            println!("Body is empty");
        }

        assert!(
            response.code >= 200 && response.code < 300,
            "Expected 2xx status code"
        );
    }

    #[test]
    fn request_with_headers() {
        let mut comm = Communicator::new().unwrap();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("X-Random".to_string(), "123".to_string());
        let request = Request::new(TEST_HOST, TEST_PORT, "GET", TEST_PATH, Some(headers), None);
        let response = comm.request(request).unwrap();

        println!("Status code: {}", response.code);
        if let Some(body) = &response.body {
            println!("{}", body);
        } else {
            println!("Body is empty");
        }

        assert!(
            response.code >= 200 && response.code < 300,
            "Expected 2xx status code"
        );
    }

    #[test]
    fn request_with_body() {
        let mut comm = Communicator::new().unwrap();
        let body = "Hello!";
        let request = Request::new(TEST_HOST, TEST_PORT, "POST", TEST_PATH, None, Some(body));
        let response = comm.request(request).unwrap();

        println!("Status code: {}", response.code);
        if let Some(body) = &response.body {
            println!("{}", body);
        } else {
            println!("Body is empty");
        }
    }
}
