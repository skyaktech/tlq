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
        let mut config = Config::default();

        if let Ok(env_value) = env::var("TLQ_PORT") {
            if let Ok(port) = env_value.parse::<u16>() {
                config.port = port;
            }
        }

        if let Ok(env_value) = env::var("TLQ_MAX_MESSAGE_SIZE") {
            if (env_value.ends_with('K') || env_value.ends_with('k')) && env_value.len() > 1 {
                if let Ok(number_of_kb) = env_value[..env_value.len() - 1].parse::<usize>() {
                    config.max_message_size = number_of_kb * 1024;
                }
            } else if let Ok(number_of_bytes) = env_value.parse::<usize>() {
                config.max_message_size = number_of_bytes;
            }
        }

        if let Ok(v) = env::var("TLQ_LOG_LEVEL") {
            config.log_level = v;
        }

        config
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
