use napi_derive::napi;

/// An HTTP request method.
#[napi]
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    /// Any method not otherwise recognized, holding its raw text (e.g.
    /// `"DELETE"`).
    Other(String),
}

impl Method {
    /// Parses a method name from a request line (e.g. `"GET"`).
    ///
    /// # Arguments
    /// - `s` — the method token as received on the wire, expected uppercase.
    ///
    /// Returns the matching [`Method`] variant, or `Method::Other(s)` if `s`
    /// isn't one of the recognized methods. Never fails.
    pub fn parse(s: &str) -> Method {
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            other => Method::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Patch => write!(f, "PATCH"),
            Method::Other(s) => write!(f, "{s}"),
        }
    }
}
