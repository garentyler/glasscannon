use glasscannon::ServerError;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let mut server = glasscannon::start().await?;
    loop {
        server.update().await?;
        // std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
