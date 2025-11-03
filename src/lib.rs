mod collector;
mod comms;

use collector::SystemFingerprint;
use comms::Communicator;
use comms::client::Request;
use windows::core::{Error, Result};

pub fn hello() -> Result<()> {
    let _fingerprint = SystemFingerprint::collect()?;
    let mut comm = Communicator::new()?;

    let args = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("example.com:443"));
    let home: Vec<&str> = args.split(":").collect();

    let hostname = home[0];
    let port: u16 = home[1].parse().unwrap();

    let request = Request::new(hostname, port, "GET", "/", None, None);

    let response = comm.request(request)?;

    if response.code != 200 {
        return Err(Error::from_win32());
    }

    println!("Response [main func]: {:?}", response.body);

    Ok(())
}
