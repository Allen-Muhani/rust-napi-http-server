#![deny(clippy::all)]

use napi::{self, Result};
use napi_derive::napi;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::http_route::{find_route, Route, RouteHandler};

mod http_header;
mod http_method;
mod http_request;
mod http_response;
mod http_route;

#[napi]
pub struct Application {
  routes: Vec<Route>,
}

#[napi]
impl Application {
  #[napi(constructor)]
  pub fn new() -> Self {
    Application { routes: Vec::new() }
  }

  #[napi]
  pub fn get(&mut self, pattern: String, handler: RouteHandler) {
    let route = Route::new("GET".to_string(), pattern, handler);
    self.routes.push(route);
  }

  #[napi]
  pub fn post(&mut self, pattern: String, handler: RouteHandler) {
    let route = Route::new("POST".to_string(), pattern, handler);
    self.routes.push(route);
  }

  #[napi]
  pub fn put(&mut self, pattern: String, handler: RouteHandler) {
    let route = Route::new("PUT".to_string(), pattern, handler);
    self.routes.push(route);
  }

  /// Binds a TCP listener on `port` and starts accepting connections in the
  /// background, dispatching each parsed request to the first registered
  /// route whose method and path pattern match.
  #[napi]
  pub fn start(&mut self, port: u16) -> Result<()> {
    let routes = Arc::new(std::mem::take(&mut self.routes));
    let addr = format!("127.0.0.1:{}", port);

    thread::spawn(move || {
      let listener = match TcpListener::bind(&addr) {
        Ok(listener) => listener,
        Err(err) => {
          eprintln!("failed to bind TCP listener on {}: {}", addr, err);
          return;
        }
      };

      println!("Application listening on {}", addr);

      for incoming in listener.incoming() {
        let stream = match incoming {
          Ok(stream) => stream,
          Err(err) => {
            eprintln!("failed to accept connection: {}", err);
            continue;
          }
        };

        let routes = Arc::clone(&routes);
        thread::spawn(move || handle_connection(stream, &routes));
      }
    });

    Ok(())
  }
}

/// Reads an incoming HTTP request from `stream`, dispatches it to the
/// matching route in `routes`, and writes the resulting response back.
///
/// # Arguments
/// - `stream` — the accepted client connection to read the request from and
///   write the response to.
/// - `routes` — the routing table to dispatch the parsed request against.
///
/// Returns nothing; the response (success, `400` on an unreadable request,
/// or `404` on no matching route) is written directly to `stream`.
fn handle_connection(stream: TcpStream, routes: &[Route]) {
  let request = match HttpRequest::from_stream(&stream) {
    Ok(Some(request)) => request,
    Ok(None) => {
      let mut response = HttpResponse::new(400, Vec::new(), String::new());
      response.send("empty request");
      return write_response(&response, &stream);
    }
    Err(e) => {
      eprintln!("Failed to read request from connection: {e}");
      let mut response = HttpResponse::new(400, Vec::new(), String::new());
      response.send(format!(
        "Bad Request: the server could not read your request. \
                 This usually means the request line or headers were \
                 malformed, or the connection closed unexpectedly while \
                 reading the body. Underlying error: {e}"
      ));
      return write_response(&response, &stream);
    }
  };

  write_response(&dispatch(routes, request), &stream);
}

/// Writes `response` back to `stream`, logging (without panicking) if the
/// write fails.
///
/// # Arguments
/// - `response` — the response to serialize as raw HTTP bytes.
/// - `stream` — the connection to write those bytes to.
///
/// Returns nothing; write failures are logged to stderr, not propagated.
fn write_response(response: &HttpResponse, stream: &TcpStream) {
  if let Err(e) = response.write_to(stream) {
    eprintln!("Failed to write response to connection: {e}");
  }
}

/// Routes `request` to the matching handler in `routes`, blocking until the
/// handler responds.
///
/// # Arguments
/// - `routes` — the routing table to search for a match.
/// - `request` — the incoming request; gains its path parameters (if any)
///   once a route matches.
///
/// Returns the handler's response on a match (or a `500` if the handler's
/// call channel failed — see `Route::call_handler`), or a `404` if nothing
/// in `routes` matches.
fn dispatch(routes: &[Route], mut request: HttpRequest) -> HttpResponse {
  match find_route(routes, &request.method(), &request.path()) {
    Ok((route, path_params)) => {
      request.set_path_params(path_params);
      match route.call_handler(request, HttpResponse::new(200, Vec::new(), String::new())) {
        Ok(handled) => handled,
        Err(e) => {
          let mut response = HttpResponse::new(500, Vec::new(), String::new());
          response.send(format!("handler error: {e}"));
          response
        }
      }
    }
    Err(error) => {
      let mut response = HttpResponse::new(404, Vec::new(), String::new());
      response.send(error);
      response
    }
  }
}
