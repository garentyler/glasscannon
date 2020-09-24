#![allow(non_snake_case)]

pub mod http;

use http::*;
use std::collections::VecDeque;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
static SRV_ID: &'static str = "GlassCannon/0.0.0";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:15000").unwrap();
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            stream.set_nodelay(true).expect("could not set nodelay");
            // stream.set_nonblocking(true).expect("could not set nonblocking");
            handle_client(stream);
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = VecDeque::from(read_bytes(&mut stream, 2048).unwrap());
}

fn read_bytes(t: &mut TcpStream, num_bytes: usize) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![];
    for _ in 0..num_bytes {
        buffer.push(0u8);
    }
    t.read(&mut buffer)?;
    Ok(buffer)
}
