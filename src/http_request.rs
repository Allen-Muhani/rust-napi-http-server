use std::collections::HashMap;

use napi_derive::napi;

#[napi(object)]
#[derive(Debug, Clone)]
pub struct Header {
  pub name: String,
  pub value: String,
}

/// A simple HTTP request struct that can be used to represent an HTTP request in a web application.
#[napi]
pub struct HttpRequest {
  /// the http method of the request (e.g. GET, POST, PUT, DELETE)
  method: String,

  /// the url of the request (e.g. /api/v1/users)
  url: String,

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
    method: String,
    url: String,
    headers: Vec<Header>,
    body: String,
    query_params: HashMap<String, String>,
    path_params: HashMap<String, String>,
  ) -> Self {
    HttpRequest {
      method,
      url,
      headers,
      body,
      query_params,
      path_params,
    }
  }

  #[napi(getter)]
  pub fn method(&self) -> String {
    self.method.clone()
  }

  #[napi(getter)]
  pub fn url(&self) -> String {
    self.url.clone()
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
  pub fn set_method(&mut self, method: String) {
    self.method = method;
  }

  #[napi(setter)]
  pub fn set_url(&mut self, url: String) {
    self.url = url;
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
