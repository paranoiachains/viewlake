#![no_main]

use viewlake::client::{Client, Request};

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    let client = Client::new().expect("Client creation error");
    let request = Request {
        hostname: "example.com",
        port: 443,
        method: "GET",
        path: "/",
        headers: Some(vec!["Header 1: asd"]),
        body: Some("Sample body"),
    };
    let handle = client.send_request(request).expect("Send request error");
    let response = client
        .receive_response(handle)
        .expect("Receive response error");

    response.read_response_into_stdout();
    0
}
