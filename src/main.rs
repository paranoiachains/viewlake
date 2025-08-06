#![no_main]

use viewlake::client::{Client, Request};

#[unsafe(no_mangle)]
pub fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    let mut client = match Client::new("Agent", None) {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Error while creating client");
            return 0;
        }
    };

    let headers: Vec<&str> = vec!["Hello: asd"];
    let request = Request::new("example.com", "GET", "/", "*/*", Some(headers), None);

    if let Err(_) = client.send_request(request) {
        eprintln!("Request failed");
        return 0;
    }

    1
}
