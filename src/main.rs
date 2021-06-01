use glasscannon::ServerError;
use log::*;

#[tokio::main]
async fn main() {
    match run_server().await {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e.message());
            std::process::exit(1);
        },
    }
}

async fn run_server() -> Result<(), ServerError> {
    let mut server = glasscannon::start().await?;
    loop {
        server.update().await?;
    }
}
