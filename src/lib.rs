pub mod http;
pub mod server;

use log::*;
pub use server::ServerError;
use server::{Config, Server};

pub static ERROR400: &str = "<!DOCTYPE html><html lang=\"en\" dir=\"ltr\"><head><meta charset=\"utf-8\"><title>400 Bad Request</title></head><body><h1>400 Bad Request</h1></body></html>";
pub static ERROR404: &str = "<!DOCTYPE html><html lang=\"en\" dir=\"ltr\"><head><meta charset=\"utf-8\"><title>404 Not Found</title></head><body><h1>404 Not Found</h1></body></html>";
pub static ERROR500: &str = "<!DOCTYPE html><html lang=\"en\" dir=\"ltr\"><head><meta charset=\"utf-8\"><title>500 Internal Server Error</title></head><body><h1>500 Internal Server Error</h1></body></html>";

pub async fn start() -> Result<Server, ServerError> {
    let config = Config::from_file("glasscannon.toml").await;
    // Set up fern logging.
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{date}][{target}][{level}] {message}",
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = record.level(),
                message = message,
            ))
        })
        .level(if cfg!(debug_assertions) {
            log::LevelFilter::Trace
        } else if let Ok(ref config) = config {
            config.loglevel
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("glasscannon.log").unwrap())
        .apply()
        .unwrap();
    if let Ok(config) = config {
        info!(
            "Starting GlassCannon v{} on port {}",
            env!("CARGO_PKG_VERSION"),
            config.port
        );
        Server::start(config).await
    } else {
        error!("Could not load config");
        Err(ServerError::ParseError)
    }
}
