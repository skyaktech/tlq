use std::env;
use std::sync::OnceLock;
use tracing::Level;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub max_message_size: usize,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 1337,
            max_message_size: 65536,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut cfg = Config::default();

        if let Ok(v) = env::var("TLQ_PORT") {
            if let Ok(p) = v.parse::<u16>() {
                cfg.port = p;
            }
        }

        if let Ok(v) = env::var("TLQ_MAX_MESSAGE_SIZE") {
            if let Ok(s) = v.parse::<usize>() {
                cfg.max_message_size = s;
            }
        }

        if let Ok(v) = env::var("TLQ_LOG_LEVEL") {
            cfg.log_level = v;
        }

        cfg
    }

    pub fn tracing_level(&self) -> Level {
        match self.log_level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::from_env)
}
