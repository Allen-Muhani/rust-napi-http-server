use napi_derive::napi;

#[napi(object)]
#[derive(Debug, Clone)]
pub struct Header {
  pub name: String,
  pub value: String,
}
