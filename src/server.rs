use crate::http::*;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    listener: TcpListener,
}
impl Server {
    pub async fn start(bind_addr: &str) -> Result<Server, ServerError> {
        Ok(Server {
            listener: TcpListener::bind(bind_addr).await?,
        })
    }
    pub async fn update(&mut self) -> Result<(), ServerError> {
        let (socket, _) = self.listener.accept().await?;
        socket.readable().await?;
        let mut data = Vec::with_capacity(4096);
        match socket.try_read_buf(&mut data) {
            Ok(0) => return Ok(()),
            Ok(_num_bytes) => {
                Server::handle_request(socket, String::from_utf8_lossy(&data).to_string()).await?
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        Ok(())
    }
    async fn handle_request(
        mut socket: TcpStream,
        request_string: String,
    ) -> Result<(), ServerError> {
        let mut response = HttpResponse::new(
            400,
            vec![HttpHeader::new("server", "GlassCannon")],
            "<html><body><h1>400 Bad Request</h1></body></html>"
                .as_bytes()
                .to_vec(),
        );
        if let Ok((_rest, _request)) = HttpRequest::parse(&request_string) {
            response = HttpResponse::new(
                200,
                vec![HttpHeader::new("server", "GlassCannon")],
                "<html><body><h1>Hello world!</h1></body></html>"
                    .as_bytes()
                    .to_vec(),
            );
        }
        socket.write_all(&response.unwrap().emit()).await?;
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
