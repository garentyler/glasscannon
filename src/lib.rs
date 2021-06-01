pub mod http;
pub mod server;

use log::info;
use server::Server;
pub use server::ServerError;

pub static BIND_ADDR: &str = "localhost:15000";

pub async fn start() -> Result<Server, ServerError> {
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
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("glasscannon.log").unwrap())
        .apply()
        .unwrap();
    info!(
        "Starting GlassCannon v{} on {}",
        env!("CARGO_PKG_VERSION"),
        BIND_ADDR
    );
    Server::start(BIND_ADDR).await
}
