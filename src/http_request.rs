
/// A simple HTTP request struct that can be used to represent an HTTP request in a web application.
#[napi(constructor)]
pub struct HttpRequest {
  /// the http method of the request (e.g. GET, POST, PUT, DELETE)
  method: String,

  /// the url of the request (e.g. /api/v1/users)
  url: String,

  /// the headers of the request (e.g. Content-Type: application/json)
  headers: Vec<(String, String)>,
  /// the body of the request (e.g. {"name": "John Doe"})
  body: String,

  /// the query parameters of the request (e.g. ?name=John&age=30)
  query_params: HashMap<String, String>,

  /// the path parameters of the request (e.g. /users/:id)
  path_params: HashMap<String, String>,
}
