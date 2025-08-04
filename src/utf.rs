pub const MAX_HEADERS: usize = 16;

pub struct Utf16Buffers {
    pub hostname: [u16; 256],
    pub hostname_len: usize,
    pub path: [u16; 256],
    pub path_len: usize,
    pub method: [u16; 256],
    pub method_len: usize,
    pub headers: [[u16; 256]; MAX_HEADERS],
    pub headers_len: [usize; MAX_HEADERS], // length for each header
    pub headers_count: usize,              // number of valid headers
    pub body: Option<([u16; 256], usize)>,
    pub accept: [u16; 256],
    pub accept_len: usize,
}

pub fn to_utf16(
    hostname: &str,
    path: &str,
    method: &str,
    headers: &[&str],
    body: Option<&str>,
    accept: &str,
) -> Utf16Buffers {
    let mut hostname_buf = [0u16; 256];
    let hostname_len = utf8_to_utf16(hostname, &mut hostname_buf);

    let mut path_buf = [0u16; 256];
    let path_len = utf8_to_utf16(path, &mut path_buf);

    let mut method_buf = [0u16; 256];
    let method_len = utf8_to_utf16(method, &mut method_buf);

    let mut headers_buf = [[0u16; 256]; MAX_HEADERS];
    let mut headers_len = [0usize; MAX_HEADERS];
    let mut headers_count = 0;

    for (i, &header) in headers.iter().enumerate().take(MAX_HEADERS) {
        headers_len[i] = utf8_to_utf16(header, &mut headers_buf[i]);
        headers_count += 1;
    }

    let body_buf = if let Some(body_str) = body {
        let mut buf = [0u16; 256];
        let len = utf8_to_utf16(body_str, &mut buf);
        Some((buf, len))
    } else {
        None
    };

    let mut accept_buf = [0u16; 256];
    let accept_len = utf8_to_utf16(accept, &mut accept_buf);

    Utf16Buffers {
        hostname: hostname_buf,
        hostname_len,
        path: path_buf,
        path_len,
        method: method_buf,
        method_len,
        headers: headers_buf,
        headers_len,
        headers_count,
        body: body_buf,
        accept: accept_buf,
        accept_len,
    }
}

pub fn utf8_to_utf16(s: &str, buf: &mut [u16]) -> usize {
    let mut i = 0;
    for c in s.encode_utf16() {
        if i == buf.len() {
            break;
        }
        buf[i] = c;
        i += 1;
    }
    i
}
