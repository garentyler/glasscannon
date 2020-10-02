#![allow(non_snake_case)]

pub mod http;

use http::*;
use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

static SRV_ID: &'static str = "GlassCannon";
static SRV_VER: &'static str = "1.0.0";

lazy_static! {
    static ref FILE: Vec<u8> = {
        let mut buf = vec![];
        get_file("./old_videos.zip")
            .expect("could not read file")
            .1
            .read_to_end(&mut buf)
            .expect("could not read file");
        buf
    };
}

fn main() {
    let _ = FILE.len(); // Load the lazy static.
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
    // Create the response.
    let mut response = HttpResponse {
        status: 200,
        headers: vec![],
        content: "".into(),
    };
    // Add the headers.
    response.set_header("Server", &format!("{}/{}", SRV_ID, SRV_VER));
    response.set_header("X-Powered-By", "RedBull and Skittles");
    response.set_header("Content-Type", "application/octet-stream");
    response.set_header("X-Litespeed-Cache", "miss");
    response.set_header("Content-Length", &FILE.len().to_string());
    response.set_header(
        "Content-Disposition",
        "attachment; filename=\"old_videos.zip\"",
    );
    // Send the response.
    stream
        .write(&Into::<String>::into(&response).into_bytes())
        .expect("could not write bytes");
    // Send the file.
    stream.write(&FILE).expect("could not write bytes");
    // Streaming version.
    // let mut num_bytes_read = 0;
    // for _ in 0..file_len {
    //     if num_bytes_read + 2048 > file_len {
    //         let mut buf = vec![];
    //         file.read_to_end(&mut buf).expect("could not read to end");
    //         stream.write(&buf).expect("could not write bytes");
    //         num_bytes_read += buf.len() as u64;
    //     } else {
    //         let mut buf = [0u8; 2048];
    //         file.read_exact(&mut buf)
    //             .expect("could not read 2048 bytes");
    //         stream.write(&buf).expect("could not write bytes");
    //         num_bytes_read += 2048;
    //     }
    // }
}

fn read_bytes(t: &mut TcpStream, num_bytes: usize) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![];
    for _ in 0..num_bytes {
        buffer.push(0u8);
    }
    t.read(&mut buffer)?;
    Ok(buffer)
}

fn get_file(path: &str) -> std::io::Result<(u64, std::fs::File)> {
    use std::fs;
    let file_len = std::fs::metadata(path)?.len();
    let file = std::fs::File::open(path)?;
    Ok((file_len, file))
}
