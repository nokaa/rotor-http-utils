use std::io;

use rotor_http::server::Response;

use file;

// TODO(nokaa): Currently any of the functions dealing with
// headers have the potential to cause a panic due to using
// `unwrap` when modifiying headers. These functions should
// probably return `Result<(), String>`, as panicing is
// undesired behavior for most web servers. This will
// greatly change the library interface, so this should
// be done before release.

static CODES: [u16; 61] = [100, 101, 102, 200, 201, 202, 203, 204, 205, 206, 207,
                           208, 226, 300, 301, 302, 303, 304, 305, 306, 307, 308,
                           400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410,
                           411, 412, 413, 414, 415, 416, 417, 418, 421, 422, 423,
                           424, 426, 428, 429, 431, 451, 500, 501, 502, 503, 504,
                           505, 506, 507, 508, 510, 511];

static MESSAGES: [&'static str; 61] = ["Continue", "Switching Protocols",
                "Processing", "OK", "Created", "Accepted", "Non-Authoritative Information",
                "No Content", "Reset Content", "Partial Content",
                "Multi-Status", "Already Reported", "IM Used",
                "Multiple Choices", "Moved Permanently", "Found", "See Other",
                "Not Modified", "Use Proxy", "Switch Proxy", "Temporary Redirect",
                "Permanent Redirect", "Bad Request", "Unauthorized",
                "Payment Required", "Forbidden", "Not Found", "Method Not Allowed",
                "Not Acceptable", "Proxy Authentication Required", "Request Timeout",
                "Conflict", "Gone", "Length Required", "Precondition Failed",
                "Payload Too Large", "URI Too Long", "Unsupported Media Type",
                "Range not Satisfiable", "Expectation Failed", "I'm a teapot",
                "Misdirected Request", "Unprocessable Entity", "Locked",
                "Failed Dependency", "Upgrade Required", "Precondition Required",
                "Too Many Requests", "Request Header Fields Too Large",
                "Unavailable For Legal Reasons", "Internal Server Error",
                "Not Implemented", "Bad Gateway", "Service Unavailable",
                "Gateway Timeout", "HTTP Version Not Supported",
                "Variant Also Negotiates", "Insufficient Storage", "Loop Detected",
                "Not Extended", "Network Authentication Required"];

/// Handles redirects. Redirects `res` to `location`
/// with code `code`. `data` should be a file/message that
/// explains what happened to the user.
/// Returns an `Err(String)` if an invalid code is received.
pub fn redirect(res: &mut Response, data: &[u8],
                location: &[u8], code: u16)
    -> Result<(), String> {
    let message = match code_lookup(code) {
        Some(m) => m,
        None => return Err("Code not found!".to_string()),
    };

    res.status(code, message);
    res.add_header("Location", location).unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();

    Ok(())
}

/// Handles sending a server error. This is useful
/// for things like a 404. `data` should explain what
/// happened to the user.
/// Returns an `Err(String)` if an invalid code is received.
pub fn error(res: &mut Response, data: &[u8], code: u16) -> Result<(), String> {
    let message = match code_lookup(code) {
        Some(m) => m,
        None => return Err("Code not found!".to_string()),
    };

    res.status(code, message);
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();

    Ok(())
}

/// Handles retrieving the message associated with a given code.
fn code_lookup<'a>(code: u16) -> Option<&'a str> {
    if let Ok(index) = CODES.binary_search(&code) {
        return Some(MESSAGES[index]);
    }

    None
}

/// Handles redirects. Redirects `res` to `location`
/// with code `code` and message `message`. `data` should be a file/message that
/// explains what happened to the user.
///
/// This function should only be used for nonstandard codes.
pub fn redirect_with_message(res: &mut Response, data: &[u8], location: &[u8],
                             code: u16, message: &str) {
    res.status(code, message);
    res.add_header("Location", location).unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();
}

/// Sends `data` to the client with status 200.
pub fn send_string(res: &mut Response, data: &[u8]) {
    res.status(200, "OK");
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();
}

/// Sends `data` to the client with status 200.
/// Sets `Content-Type` header to `text/plain`.
pub fn send_string_raw(res: &mut Response, data: &[u8]) {
    res.status(200, "OK");
    // Add `Content-Type` header to ensure data is interpreted
    // as plaintext
    res.add_header("Content-Type",
                   "text/plain; charset=utf-8".as_bytes())
        .unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();
}

/// Sends data read from `filename` to the client
/// with status 200.
///
/// Returns `Err(io::Error)` if `filename` cannot be read.
pub fn send_file(res: &mut Response, filename: &str) -> Result<(), io::Error> {
    let data = &try!(file::read_file(filename))[..];

    res.status(200, "OK");
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();

    Ok(())
}

/// Sends data read from `filename` to the client
/// with status 200. This is used to send a file as
/// plain text.
/// Sets `Content-Type` header to `text/plain`.
///
/// Returns `Err(io::Error)` if `filename` cannot be read.
pub fn send_file_text(res: &mut Response, filename: &str) -> Result<(), io::Error> {
    let data = &try!(file::read_file(filename))[..];

    res.status(200, "OK");
    // Add `Content-Type` header to ensure data is interpreted
    // as plaintext
    res.add_header("Content-Type",
                   "text/plain; charset=utf-8".as_bytes())
        .unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();

    Ok(())
}

/// Sends data read from `filename` to the client
/// with status 200. This is used for sending a file
/// to the user for download.
/// Sets `Content-Type` header to `application/octet-stream`.
///
/// Returns `Err(io::Error)` if `filename` cannot be read.
pub fn send_file_raw(res: &mut Response, filename: &str) -> Result<(), io::Error> {
    let data = &try!(file::read_file(filename))[..];

    res.status(200, "OK");
    // Add `Content-Type` header to ensure data is interpreted
    // as plaintext
    res.add_header("Content-Type",
                   "application/octet-stream; charset=utf-8".as_bytes())
        .unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
    res.done();

    Ok(())
}
