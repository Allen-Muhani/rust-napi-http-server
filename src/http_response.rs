use napi_derive::napi;
use std::io::{self, Write};

use crate::http_header::Header;

/// A plain-data HTTP response: a `#[napi(object)]`, not a `#[napi]` class,
/// so it converts to/from JS by value (a real `FromNapiValue` impl) rather
/// than by reference. That's required for `RouteHandler` (src/http_route.rs)
/// to get a genuine owned `HttpResponse` back out of a JS handler's return
/// value — `#[napi]` classes only support owned-value conversion one way
/// (Rust -> JS), so a class-typed `HttpResponse` could never make it back
/// across a `ThreadsafeFunction` call synchronously. JS mutates this like
/// any plain object (`res.statusCode = ...`) and must `return res;`.
#[napi(object)]
#[derive(Clone)]
pub struct HttpResponse {
  pub status_code: u16,
  pub headers: Vec<Header>,
  pub body: String,
}

impl HttpResponse {
  pub fn new(status_code: u16, headers: Vec<Header>, body: String) -> Self {
    HttpResponse {
      status_code,
      headers,
      body,
    }
  }
}

impl HttpResponse {
  /// Writes this response as raw HTTP bytes to `writer` (e.g. a `TcpStream`).
  ///
  /// # Arguments
  /// - `writer` — the destination to write the status line, headers, and
  ///   body to.
  ///
  /// Returns `Ok(())` on success, or `Err` if the underlying write fails.
  pub fn write_to(&self, mut writer: impl Write) -> io::Result<()> {
    let mut head = format!(
      "HTTP/1.1 {} {}\r\n",
      self.status_code,
      status_reason(self.status_code)
    );
    for header in &self.headers {
      head.push_str(&format!("{}: {}\r\n", header.name, header.value));
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", self.body.len()));

    writer.write_all(head.as_bytes())?;
    writer.write_all(self.body.as_bytes())
  }

  /// Sets the response body, converting `body` into a `String`.
  ///
  /// # Arguments
  /// - `body` — the raw response body text.
  ///
  /// Returns nothing; sets `self.body`.
  pub fn send(&mut self, body: impl Into<String>) {
    self.body = body.into();
  }
}

/// Maps a status code to its standard reason phrase (e.g. `200` -> `"OK"`).
///
/// # Arguments
/// - `status` — the status code to look up.
///
/// Returns the reason phrase, or `"Unknown"` if `status` isn't one of the
/// codes this server sends.
fn status_reason(status: u16) -> &'static str {
  match status {
    200 => "OK",
    201 => "Created",
    400 => "Bad Request",
    404 => "Not Found",
    500 => "Internal Server Error",
    _ => "Unknown",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn write_to_formats_status_line_headers_and_body() {
    let response = HttpResponse::new(
      200,
      vec![Header {
        name: "Content-Type".to_string(),
        value: "text/plain".to_string(),
      }],
      "hello".to_string(),
    );

    let mut buf = Vec::new();
    response.write_to(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    assert_eq!(
      text,
      "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello"
    );
  }

  #[test]
  fn write_to_uses_unknown_reason_for_unrecognized_status() {
    let response = HttpResponse::new(999, Vec::new(), String::new());

    let mut buf = Vec::new();
    response.write_to(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    assert!(text.starts_with("HTTP/1.1 999 Unknown\r\n"));
  }

  #[test]
  fn send_overwrites_body() {
    let mut response = HttpResponse::new(200, Vec::new(), "old".to_string());
    response.send("new");
    assert_eq!(response.body, "new");
  }
}
