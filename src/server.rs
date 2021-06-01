use crate::http::*;
use log::*;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;

pub struct Server {
    listener: TcpListener,
    resources: HashMap<String, Vec<u8>>,
}
impl Server {
    pub async fn start(bind_addr: &str) -> Result<Server, ServerError> {
        let mut resources = HashMap::new();
        let mut file_paths = vec![];
        let mut entries = tokio::fs::read_dir("./res/").await?;
        while let Some(entry) = entries.next_entry().await? {
            file_paths.push(entry.path());
        }
        for path in &file_paths {
            let mut contents = vec![];
            File::open(path).await?.read_to_end(&mut contents).await?;
            let trimmed_path = (&path.to_str().unwrap()[5..]).to_owned();
            resources.insert(trimmed_path, contents);
        }
        Ok(Server {
            listener: TcpListener::bind(bind_addr).await?,
            resources,
        })
    }
    pub async fn update(&mut self) -> Result<(), ServerError> {
        let (socket, _) = self.listener.accept().await?;
        socket.readable().await?;
        let mut data = Vec::with_capacity(4096);
        match socket.try_read_buf(&mut data) {
            Ok(0) => return Ok(()),
            Ok(_num_bytes) => {
                self.handle_request(socket, String::from_utf8_lossy(&data).to_string()).await?
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        Ok(())
    }
    async fn handle_request(
        &self,
        mut socket: TcpStream,
        request_string: String,
    ) -> Result<(), ServerError> {
        let mut response = HttpResponse::new(
            400,
            vec![],
            self.resources.get(&"/400.html".to_owned()).unwrap_or(&crate::ERROR400.as_bytes().to_vec()).clone(),
        ).expect("could not load 400 page");
        if let Ok((_rest, request)) = HttpRequest::parse(&request_string) {
            if self.resources.contains_key(request.path.path()) {
                response = HttpResponse::new(
                    200,
                    vec![],
                    self.resources.get(request.path.path()).unwrap().clone(),
                ).expect("bad request");
            } else {
                response = HttpResponse::new(
                    404,
                    vec![],
                    self.resources.get(&"/404.html".to_owned()).unwrap_or(&crate::ERROR404.as_bytes().to_vec()).clone(),
                ).expect("could not load 404 page");
            }
            // info!("{} {} {}", response.status.value, request.method, request.path.path());
        }
        response.headers.push(HttpHeader::new("server", "GlassCannon"));
        socket.write_all(&response.emit()).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ServerError {
    IoError(std::io::Error),
    ParseError,
}
impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> ServerError {
        ServerError::IoError(error)
    }
}
impl<A> From<nom::Err<A>> for ServerError {
    fn from(_error: nom::Err<A>) -> ServerError {
        ServerError::ParseError
    }
}
