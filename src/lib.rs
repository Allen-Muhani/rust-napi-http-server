#![deny(clippy::all)]

use napi::{self, Result};
use napi_derive::napi;
use std::io::Read;
use std::net::TcpListener;
use std::thread;


#[napi]
pub fn start_tcp_listener(port: u16) -> Result<()> {
  let addr = format!("127.0.0.1:{}", port);

  thread::spawn(move || {
    let listener = match TcpListener::bind(&addr) {
      Ok(listener) => listener,
      Err(err) => {
        eprintln!("failed to bind TCP listener on {}: {}", addr, err);
        return;
      }
    };

    println!("TCP listener started on {}", addr);

    for incoming in listener.incoming() {
      match incoming {
        Ok(mut stream) => {
          let mut buffer = [0u8; 4096];
          match stream.read(&mut buffer) {
            Ok(0) => {
              println!("connection closed without data");
            }
            Ok(n) => {
              let data = &buffer[..n];
              let text = String::from_utf8_lossy(data);
              println!("received {} bytes: {}", n, text);
            }
            Err(err) => {
              eprintln!("failed to read from stream: {}", err);
            }
          }
        }
        Err(err) => {
          eprintln!("failed to accept connection: {}", err);
        }
      }
    }
  });

  Ok(())
}
