use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Result};
use std::net::TcpStream;

use napi_derive::napi;

use crate::http_header::Header;
use crate::http_method::Method;

/// A simple HTTP request struct that can be used to represent an HTTP request in a web application.
#[napi]
pub struct HttpRequest {
  /// the http method of the request (e.g. GET, POST, PUT, DELETE)
  method: Method,

  /// the url of the request (e.g. /api/v1/users)
  path: String,

  /// the headers of the request (e.g. Content-Type: application/json)
  headers: Vec<Header>,
  /// the body of the request (e.g. {"name": "John Doe"})
  body: String,

  /// the query parameters of the request (e.g. ?name=John&age=30)
  query_params: HashMap<String, String>,

  /// the path parameters of the request (e.g. /users/:id)
  path_params: HashMap<String, String>,
}

#[napi]
impl HttpRequest {
  #[napi(constructor)]
  pub fn new(
    method: Method,
    path: String,
    headers: Vec<Header>,
    body: String,
    query_params: HashMap<String, String>,
    path_params: HashMap<String, String>,
  ) -> Self {
    HttpRequest {
      method,
      path,
      headers,
      body,
      query_params,
      path_params,
    }
  }

  #[napi(getter)]
  pub fn method(&self) -> Method {
    self.method.clone()
  }

  #[napi(getter)]
  pub fn path(&self) -> String {
    self.path.clone()
  }

  #[napi(getter)]
  pub fn headers(&self) -> Vec<Header> {
    self.headers.clone()
  }

  #[napi(getter)]
  pub fn body(&self) -> String {
    self.body.clone()
  }

  #[napi(getter)]
  pub fn query_params(&self) -> HashMap<String, String> {
    self.query_params.clone()
  }

  #[napi(getter)]
  pub fn path_params(&self) -> HashMap<String, String> {
    self.path_params.clone()
  }

  #[napi(setter)]
  pub fn set_method(&mut self, method: Method) {
    self.method = method;
  }

  #[napi(setter)]
  pub fn set_path(&mut self, path: String) {
    self.path = path;
  }

  #[napi(setter)]
  pub fn set_headers(&mut self, headers: Vec<Header>) {
    self.headers = headers;
  }

  #[napi(setter)]
  pub fn set_body(&mut self, body: String) {
    self.body = body;
  }

  #[napi(setter)]
  pub fn set_query_params(&mut self, query_params: HashMap<String, String>) {
    self.query_params = query_params;
  }

  #[napi(setter)]
  pub fn set_path_params(&mut self, path_params: HashMap<String, String>) {
    self.path_params = path_params;
  }
}

impl HttpRequest {
  /// Parses an `HttpRequest` out of a readable stream (e.g. a `TcpStream`).
  ///
  /// Reads the request line (`METHOD /path?query HTTP/1.1`), then header
  /// lines up to the blank line that separates headers from the body, then
  /// reads exactly `Content-Length` bytes of body if that header was sent.
  /// Query parameters on the path (e.g. `?id=5`) are parsed into `params`.
  ///
  /// # Arguments
  /// - `stream` — the client connection to read the request from.
  ///
  /// Returns `Ok(Some(request))` on a successfully parsed request,
  /// `Ok(None)` if the connection had no request line to read (e.g. the
  /// client closed the connection immediately or sent a blank line), or
  /// `Err` if reading from `stream` fails or a header line isn't valid
  /// UTF-8.
  pub fn from_stream(stream: &TcpStream) -> Result<Option<Self>> {
    let mut reader = BufReader::new(stream);
    let mut lines = reader.by_ref().lines();

    // The request line is the first line, e.g. "GET /users?id=5 HTTP/1.1".
    // Its absence (empty read or EOF) means there's no request to parse.

    let request_line = match lines.next() {
      Some(Ok(line)) => line,
      Some(Err(e)) => {
        eprintln!("Failed to read line from connection: {e}");
        return Err(e);
      }
      None => return Ok(None),
    };
    if request_line.is_empty() {
      return Ok(None);
    }

    let mut parts = request_line.split_whitespace();
    let method = Method::parse(parts.next().unwrap_or(""));
    let raw_path = parts.next().unwrap_or("");

    // Split the query string (if any) off the path so `path` stays a
    // plain route like "/users" and query params land in `query_params`.
    let (path, query_params) = match raw_path.split_once('?') {
      Some((path, query)) => (path.to_string(), parse_query_string(query)),
      None => (raw_path.to_string(), HashMap::new()),
    };

    let (headers, content_length) = match parse_headers(&mut lines) {
      Ok((headers, content_length)) => (headers, content_length),
      Err(e) => {
        eprintln!("Failed to parse headers from connection: {e}");
        return Err(e);
      }
    };
    let body = read_body(&mut reader, content_length)?;

    Ok(Some(HttpRequest {
      method,
      path,
      headers,
      body,
      query_params,
      path_params: HashMap::new(),
    }))
  }

  /// Deserializes the request body as JSON into `T`.
  ///
  /// Takes no arguments beyond `self` (deserializes `self.body`). Returns
  /// `Some(T)` on success, or `None` if the body is empty, not valid JSON,
  /// or doesn't match the shape of `T`.
  pub fn json<T: DeserializeOwned>(&self) -> Option<T> {
    serde_json::from_str(&self.body).ok()
  }
}

/// Parses a URL query string (e.g. `id=5&sort=asc`) into name/value pairs.
///
/// # Arguments
/// - `query` — the raw query string, without the leading `?`.
///
/// Returns the parsed name/value pairs. Entries without an `=` are skipped
/// rather than treated as an error; never fails.
fn parse_query_string(query: &str) -> HashMap<String, String> {
  println!("Parsing query string: {query}");
  query
    .split('&')
    .filter_map(|pair| pair.split_once('='))
    .map(|(name, value)| (name.to_string(), value.to_string()))
    .collect()
}

/// Reads header lines from `lines` until the blank line that ends them.
///
/// # Arguments
/// - `lines` — an iterator over the raw lines following the request line,
///   not yet consumed past the header block.
///
/// Returns `Ok` with the collected `(name, value)` headers, in the order
/// received, and the `Content-Length` value seen (`0` if the header wasn't
/// present or didn't parse as a number). Returns `Err` if reading a line
/// from `lines` fails.
fn parse_headers(lines: &mut impl Iterator<Item = Result<String>>) -> Result<(Vec<Header>, usize)> {
  let mut headers: Vec<Header> = Vec::new();
  let mut content_length: usize = 0;
  for line in lines {
    let line = match line {
      Ok(line) => line,
      Err(e) => {
        eprintln!("Failed to read header line from connection: {e}");
        return Err(e);
      }
    };
    if line.is_empty() {
      break;
    }
    if let Some((name, value)) = line.split_once(": ") {
      if name.eq_ignore_ascii_case("Content-Length") {
        content_length = value.trim().parse().unwrap_or(0);
      }
      headers.push(Header {
        name: name.to_string(),
        value: value.to_string(),
      });
    }
  }
  Ok((headers, content_length))
}

/// Reads exactly `content_length` bytes from `reader` as the request body.
///
/// # Arguments
/// - `reader` — the stream positioned right after the header block.
/// - `content_length` — the byte count from the `Content-Length` header.
///
/// Returns `Ok("")` without reading anything if `content_length` is `0`, so
/// bodyless requests (e.g. `GET`) never block waiting for bytes that will
/// never arrive. Otherwise returns `Ok` with the body decoded as UTF-8
/// (lossily, replacing invalid sequences), or `Err` if the read fails.
fn read_body(reader: &mut impl Read, content_length: usize) -> Result<String> {
  if content_length == 0 {
    return Ok(String::new());
  }

  let mut buf = vec![0u8; content_length];
  match reader.read_exact(&mut buf) {
    Ok(()) => Ok(String::from_utf8_lossy(&buf).into_owned()),
    Err(e) => {
      eprintln!("Failed to read request body from connection: {e}");
      Err(e)
    }
  }
}
