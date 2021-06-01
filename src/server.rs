use crate::http::*;
use log::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[async_recursion::async_recursion]
async fn get_files(dir: PathBuf, config: &Config) -> Result<Vec<String>, ServerError> {
    let mut file_paths = vec![];
    let mut entries = tokio::fs::read_dir(dir.as_path()).await?;
    while let Some(entry) = entries.next_entry().await? {
        file_paths.push(entry.path());
    }
    let mut recursive_paths = vec![];
    for path in &file_paths {
        if path.as_path().is_file() {
            recursive_paths.push(String::from(path.to_str().unwrap_or("ERR")));
        } else if path.as_path().is_dir() {
            recursive_paths.append(&mut get_files(path.clone(), config).await?);
        }
    }
    let mut set = HashSet::new();
    recursive_paths.retain(|x| set.insert(x.clone()));
    recursive_paths.retain(|x| config.preload.contains(&String::from(&x[5..])));
    Ok(recursive_paths)
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub port: u16,
    pub resources: PathBuf,
    pub preload: Vec<String>,
    pub mimetypes: HashMap<String, String>,
    pub loglevel: log::LevelFilter,
}
impl Config {
    pub fn new(
        port: u16,
        resources: PathBuf,
        preload: Vec<String>,
        mimetypes: HashMap<String, String>,
        loglevel: log::LevelFilter,
    ) -> Config {
        Config {
            port,
            resources,
            preload,
            mimetypes,
            loglevel,
        }
    }
    pub async fn from_file(path: &str) -> Result<Config, ServerError> {
        let mut port = 15000;
        let mut resources = PathBuf::from("./res/");
        let mut preload = vec![];
        let mut mimetypes = HashMap::new();
        let mut loglevel = log::LevelFilter::Info;
        if Path::new(path).exists() {
            use toml::Value;
            let mut contents = vec![];
            File::open(&path).await?.read_to_end(&mut contents).await?;
            if let Value::Table(cfg) = String::from_utf8_lossy(&contents).parse::<Value>()? {
                if let Some(Value::Table(cfg_server)) = cfg.get("server") {
                    if let Some(Value::Integer(cfg_port)) = cfg_server.get("port") {
                        port = *cfg_port as u16;
                    }
                    if let Some(Value::String(cfg_resources)) = cfg_server.get("resources") {
                        resources = PathBuf::from(cfg_resources.clone());
                    }
                    if let Some(Value::Array(cfg_preload)) = cfg_server.get("preload") {
                        for preloaded in cfg_preload {
                            if let Value::String(cfg_preloaded) = preloaded {
                                preload.push(cfg_preloaded.clone());
                            }
                        }
                    }
                    if let Some(Value::String(cfg_loglevel)) = cfg_server.get("loglevel") {
                        match &cfg_loglevel[..] {
                            "none" => loglevel = log::LevelFilter::Off,
                            "error" => loglevel = log::LevelFilter::Error,
                            "warn" => loglevel = log::LevelFilter::Warn,
                            "info" => loglevel = log::LevelFilter::Info,
                            "debug" => loglevel = log::LevelFilter::Debug,
                            "trace" | "all" => loglevel = log::LevelFilter::Trace,
                            _ => {}
                        }
                    }
                }
                if let Some(Value::Table(cfg_mimetypes)) = cfg.get("mimetypes") {
                    for k in cfg_mimetypes.keys() {
                        if let Some(Value::Array(cfg_mimetype)) = cfg_mimetypes.get(k) {
                            for extension in cfg_mimetype {
                                if let Value::String(cfg_extension) = extension {
                                    mimetypes.insert(cfg_extension.clone(), k.clone());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            warn!("No config file detected! Creating one at glasscannon.toml...");
            File::create(&path).await?.write_all(b"[server]\nport = 15000\nresources = \"./res/\"\npreload = []\nloglevel = \"info\" # See https://docs.rs/log/0.4.14/log/enum.LevelFilter.html\n\n[mimetypes]\n\"text/html\" = \"html\"").await?;
        }
        Ok(Config::new(port, resources, preload, mimetypes, loglevel))
    }
}

pub struct Server {
    listener: TcpListener,
    resources: HashMap<String, Option<Vec<u8>>>,
    config: Config,
}
impl Server {
    pub async fn start(config: Config) -> Result<Server, ServerError> {
        let mut resources = HashMap::new();
        for path in get_files(PathBuf::from("./res/"), &config).await? {
            let mut contents = vec![];
            let trimmed_path = (&path.clone().as_str()[5..]).to_owned();
            if config.preload.contains(&trimmed_path) {
                File::open(&path).await?.read_to_end(&mut contents).await?;
                resources.insert(trimmed_path, Some(contents));
            } else {
                resources.insert(trimmed_path, None);
            }
        }
        Ok(Server {
            listener: TcpListener::bind(format!("localhost:{}", config.port)).await?,
            resources,
            config,
        })
    }
    pub async fn update(&mut self) -> Result<(), ServerError> {
        let (socket, _) = self.listener.accept().await?;
        socket.readable().await?;
        let mut data = Vec::with_capacity(4096);
        match socket.try_read_buf(&mut data) {
            Ok(0) => return Ok(()),
            Ok(_num_bytes) => {
                self.handle_request(socket, String::from_utf8_lossy(&data).to_string())
                    .await?
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
        let mut response = HttpResponse::new()
            .status(400)
            .header("Content-Type", "text/html")
            .body(
                self.resources
                    .get(&"/400.html".to_owned())
                    .unwrap_or(&Some(crate::ERROR400.as_bytes().to_vec()))
                    .clone()
                    .unwrap(),
            )
            .build();
        if let Ok((_rest, request)) = HttpRequest::parse(&request_string) {
            let mut url_path = request.path.path().clone();
            if request.path.path() == "/" {
                url_path = "/index.html";
            }
            let mut file_path = self.config.resources.clone();
            file_path.push(&url_path[1..]);
            if self.resources.contains_key(request.path.path()) {
                response = HttpResponse::new()
                    .status(200)
                    .body(
                        self.resources
                            .get(request.path.path())
                            .unwrap()
                            .clone()
                            .unwrap(),
                    )
                    .build();
                if let Some(ext) = Path::new(url_path).extension() {
                    if let Some(mime) = self.config.mimetypes.get(ext.to_str().unwrap()) {
                        response.set_header("Content-Type", mime);
                    }
                } else {
                    response.set_header("Content-Type", "application/octet-stream");
                }
            } else if file_path.as_path().exists() {
                let mut contents = vec![];
                File::open(file_path)
                    .await?
                    .read_to_end(&mut contents)
                    .await?;
                response = HttpResponse::new().status(200).body(contents).build();
                if let Some(ext) = Path::new(url_path).extension() {
                    if let Some(mime) = self.config.mimetypes.get(ext.to_str().unwrap()) {
                        response.set_header("Content-Type", mime);
                    }
                } else {
                    response.set_header("Content-Type", "application/octet-stream");
                }
            } else {
                response = HttpResponse::new()
                    .status(404)
                    .header("Content-Type", "text/html")
                    .body(
                        self.resources
                            .get(&"/404.html".to_owned())
                            .unwrap_or(&Some(crate::ERROR404.as_bytes().to_vec()))
                            .clone()
                            .unwrap(),
                    )
                    .build();
            }
            info!(
                "{} {} {}",
                response.status.value,
                request.method,
                request.path.path()
            );
        }
        response.set_header("server", "GlassCannon");
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
impl From<toml::de::Error> for ServerError {
    fn from(_error: toml::de::Error) -> ServerError {
        ServerError::ParseError
    }
}
