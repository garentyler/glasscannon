#![allow(non_snake_case)]

pub mod http;

use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
static SRV_ID: &'static str = "GlassCannon/0.0.0";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:15000").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        hc(stream);
    }
}

fn hc(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let get = b"GET / HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get) {
        (sc("200".to_string()), "hello.html")
    } else {
        ("HTTP/1.1 405 METHOD NOT ALLOWED\r\n\r\n", "405.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn sc(code: String) -> &'static str {
    if code.eq("200") {
        return "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\nserver: GlassCannon\r\n\r\n";
    }
    return "HTTP/1.1 400 BAD REQUEST\r\nserver: GlassCannon\r\n\r\n";
}
