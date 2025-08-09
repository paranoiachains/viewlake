#![no_main]

use viewlake::client::{Client, Request};

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

#[unsafe(no_mangle)]
unsafe extern "C" fn main(_argc: i32, _argv: *const *const u8) -> u32 {
    let mut client = Client::new().expect("Client creation error");
    let request = build_request(
        "httpbin.org",
        443,
        "GET",
        "/anything",
        Some(vec!["Header1: 1", "Header2: 2"]),
        Some("Body"),
    );

    let mut handle = client.send_request(request).expect("Send request error");
    let response = client
        .receive_response(&handle)
        .expect("Receive response error");

    response.into_stdout();
    0
}
