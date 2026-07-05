use std::collections::HashMap;
use std::sync::mpsc;

use napi::bindgen_prelude::FnArgs;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use crate::http_method::Method;
use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;

/// Maps an HTTP method and path pattern to the handler that serves it.
#[napi]
pub struct Route {
  /// The HTTP method this route matches (e.g. `"GET"`).
  pub method: String,
  /// The path pattern this route matches, e.g. `"/users"` or, with a path
  /// parameter, `"/location/:id"`.
  pub pattern: String,
  /// The handler invoked when this route matches an incoming request.
  ///
  /// Not exposed as a napi getter/setter: `ThreadsafeFunction` only
  /// implements `FromNapiValue` (JS -> Rust), not `ToNapiValue`, so it can
  /// only be accepted through the constructor, never read back out to JS.
  handler: RouteHandler,
}

// `FnArgs<(A, B)>` (not a bare `(A, B)` tuple) is what makes napi spread the
// two values into two separate JS parameters `(req, res) => ...`; a bare
// tuple converts as a single value instead, per the blanket
// `impl<T: ToNapiValue> JsValuesTupleIntoVec for T`. `T` and `CallJsBackArgs`
// must be the same type here — `ThreadsafeFunction`'s only `FromNapiValue`
// impl (needed to accept the JS function passed into `Route::new`) requires
// it.
#[napi]
pub type RouteHandler = ThreadsafeFunction<
  FnArgs<(HttpRequest, HttpResponse)>, // T: args Rust sends
  HttpResponse,                        // Return: the (possibly mutated) response JS hands back
  FnArgs<(HttpRequest, HttpResponse)>, // CallJsBackArgs: what JS receives, spread into (req, res)
  napi::Status,                        // Status type
  false,                               // CalleeHandled = false → no (err, ...) first arg
>;

#[napi]
impl Route {
  #[napi(constructor)]
  pub fn new(method: String, pattern: String, handler: RouteHandler) -> Self {
    Route {
      method,
      pattern,
      handler,
    }
  }
}

impl Route {
  /// Invokes this route's handler with `request`/`response` and blocks the
  /// calling thread until the JS handler resolves.
  ///
  /// The JS handler receives `(request, response)`, may mutate `response`
  /// like any plain object (e.g. `res.statusCode = 200`), and must `return`
  /// it (or a fresh response) — that return value is what flows back here.
  /// `HttpResponse` is a `#[napi(object)]` plain-data type specifically so
  /// this round trip has a real `FromNapiValue` to land on; a `#[napi]`
  /// class couldn't make this trip synchronously.
  ///
  /// # Arguments
  /// - `request` — the parsed request to hand to the JS handler.
  /// - `response` — the response to hand to the JS handler for mutation.
  ///
  /// Returns the `HttpResponse` the JS handler returned, or `Err` if the
  /// handler's call channel closed before it responded (e.g. the handler
  /// threw or the tsfn was aborted).
  pub fn call_handler(&self, request: HttpRequest, response: HttpResponse) -> napi::Result<HttpResponse> {
    let (tx, rx) = mpsc::channel();
    self.handler.call_with_return_value(
      FnArgs::from((request, response)),
      ThreadsafeFunctionCallMode::NonBlocking,
      move |result, _env| {
        let _ = tx.send(result);
        Ok(())
      },
    );
    rx.recv().unwrap_or_else(|_| {
      Err(napi::Error::from_reason(
        "handler channel closed before responding",
      ))
    })
  }
}

impl Route {
  /// Checks whether `method` matches this route's method.
  ///
  /// # Arguments
  /// - `method` — the incoming request's parsed method.
  ///
  /// Returns `true` if it equals this route's `method` field (parsed the
  /// same way), `false` otherwise.
  pub fn matches_method(&self, method: &Method) -> bool {
    Method::parse(&self.method) == *method
  }

  /// Checks whether `path` matches this route's pattern, independent of
  /// method.
  ///
  /// `path` is matched against `pattern` segment by segment: a pattern
  /// segment starting with `:` (e.g. `:id`) captures whatever segment is
  /// in that position of `path`.
  ///
  /// # Arguments
  /// - `path` — the incoming request's path, e.g. `"/location/5"`.
  ///
  /// Returns `Some` map of captured path parameter names to their values
  /// on a match (empty if the pattern had no `:param` segments), or `None`
  /// if the segment count or a literal segment differs.
  pub fn matches_path(&self, path: &str) -> Option<HashMap<String, String>> {
    // Filtering out empty segments means "/user", "/user/", and
    // "/user//" (or a doubled slash anywhere) all match the same way,
    // instead of a stray "/" changing the segment count.
    let pattern_segments: Vec<&str> = self.pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_segments.len() != path_segments.len() {
      return None;
    }

    let mut params = HashMap::new();
    for (pattern, path) in pattern_segments.iter().zip(path_segments.iter()) {
      if pattern == path {
        continue;
      }
      match pattern.strip_prefix(':') {
        Some(name) => {
          params.insert(name.to_string(), path.to_string());
        }
        None => return None,
      }
    }

    Some(params)
  }
}

/// Finds the first route in `routes` matching `method` and `path`.
///
/// # Arguments
/// - `routes` — the routing table to search, in priority order.
/// - `method` — the incoming request's parsed method.
/// - `path` — the incoming request's path (query string already stripped).
///
/// Returns `Ok` with the matched route and its captured path parameters
/// (empty if the pattern had none), or `Err` with a message if no route in
/// `routes` matches both `method` and `path`.
pub fn find_route<'a>(
  routes: &'a [Route],
  method: &Method,
  path: &str,
) -> Result<(&'a Route, HashMap<String, String>), String> {
  routes
    .iter()
    .filter(|route| route.matches_method(method))
    .find_map(|route| route.matches_path(path).map(|params| (route, params)))
    .ok_or_else(|| "Route not found".to_string())
}
