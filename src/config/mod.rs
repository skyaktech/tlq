use std::env;
use std::sync::OnceLock;
use tracing::Level;

const DEFAULT_PORT: u16 = 1337;
const DEFAULT_MAX_MESSAGE_SIZE: usize = 65536; // 64KB
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub max_message_size: usize,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            log_level: DEFAULT_LOG_LEVEL.to_string(),
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
            if let Some(size) = Self::parse_size(&env_value) {
                config.max_message_size = size;
            }
        }

        if let Ok(v) = env::var("TLQ_LOG_LEVEL") {
            config.log_level = v;
        }

        config
    }

    fn parse_size(value: &str) -> Option<usize> {
        if value.is_empty() {
            return None;
        }

        if let Some(kb_str) = value.strip_suffix(['K', 'k']) {
            kb_str
                .parse::<usize>()
                .ok()
                .filter(|&kb| kb > 0)
                .map(|kb| kb * 1024)
        } else {
            value.parse::<usize>().ok().filter(|&bytes| bytes > 0)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Ensure tests don't run in parallel and interfere with each other's env vars
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn with_env_var<F>(key: &str, value: &str, test: F)
    where
        F: FnOnce(),
    {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();
        env::set_var(key, value);
        test();
        clear_env_vars();
    }

    fn clear_env_vars() {
        env::remove_var("TLQ_PORT");
        env::remove_var("TLQ_MAX_MESSAGE_SIZE");
        env::remove_var("TLQ_LOG_LEVEL");
    }

    #[test]
    fn test_default_config() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();
        let config = Config::from_env();
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.max_message_size, DEFAULT_MAX_MESSAGE_SIZE);
        assert_eq!(config.log_level, DEFAULT_LOG_LEVEL);
    }

    #[test]
    fn test_ports() {
        let test_cases = vec![
            ("8080", 8080, "valid port"),
            ("not-a-port", DEFAULT_PORT, "invalid string"),
            ("99999", DEFAULT_PORT, "out of range"),
            ("", DEFAULT_PORT, "empty string"),
        ];

        for (input, expected_port, description) in test_cases {
            with_env_var("TLQ_PORT", input, || {
                let config = Config::from_env();
                assert_eq!(
                    config.port, expected_port,
                    "Failed for {}: input '{}'",
                    description, input
                );
            });
        }
    }

    #[test]
    fn test_message_sizes() {
        let test_cases = vec![
            ("1024", 1024, "raw bytes"),
            ("64K", 64 * 1024, "uppercase K suffix"),
            ("128k", 128 * 1024, "lowercase k suffix"),
            ("abc", DEFAULT_MAX_MESSAGE_SIZE, "invalid format"),
            ("K", DEFAULT_MAX_MESSAGE_SIZE, "just K"),
            ("0", DEFAULT_MAX_MESSAGE_SIZE, "zero value"),
            ("0k", DEFAULT_MAX_MESSAGE_SIZE, "zero with k suffix"),
            ("", DEFAULT_MAX_MESSAGE_SIZE, "empty string"),
        ];

        for (input, expected_size, description) in test_cases {
            with_env_var("TLQ_MAX_MESSAGE_SIZE", input, || {
                let config = Config::from_env();
                assert_eq!(
                    config.max_message_size, expected_size,
                    "Failed for {}: input '{}'",
                    description, input
                );
            });
        }
    }
    #[test]
    fn test_log_levels() {
        let test_cases = vec![
            ("trace", Level::TRACE),
            ("debug", Level::DEBUG),
            ("info", Level::INFO),
            ("warn", Level::WARN),
            ("warning", Level::WARN),
            ("error", Level::ERROR),
            ("INFO", Level::INFO),
            ("Info", Level::INFO),
            ("invalid", Level::INFO),
            ("", Level::INFO),
        ];

        for (input, expected_level) in test_cases {
            with_env_var("TLQ_LOG_LEVEL", input, || {
                let config = Config::from_env();
                assert_eq!(config.log_level, input);
                assert_eq!(
                    config.tracing_level(),
                    expected_level,
                    "Failed for log level: {}",
                    input
                );
            });
        }
    }

    #[test]
    fn test_multiple_env_vars() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_env_vars();
        env::set_var("TLQ_PORT", "3000");
        env::set_var("TLQ_MAX_MESSAGE_SIZE", "32K");
        env::set_var("TLQ_LOG_LEVEL", "debug");

        let config = Config::from_env();
        assert_eq!(config.port, 3000);
        assert_eq!(config.max_message_size, 32 * 1024);
        assert_eq!(config.log_level, "debug");

        clear_env_vars();
    }

    #[test]
    fn test_partial_env_vars() {
        with_env_var("TLQ_PORT", "5000", || {
            let config = Config::from_env();
            assert_eq!(config.port, 5000);
            assert_eq!(config.max_message_size, DEFAULT_MAX_MESSAGE_SIZE);
            assert_eq!(config.log_level, DEFAULT_LOG_LEVEL);
        });
    }

    #[test]
    fn test_parse_size_helper() {
        // Valid cases
        assert_eq!(Config::parse_size("1024"), Some(1024));
        assert_eq!(Config::parse_size("64K"), Some(65536));
        assert_eq!(Config::parse_size("64k"), Some(65536));
        assert_eq!(Config::parse_size("1K"), Some(1024));

        // Invalid cases
        assert_eq!(Config::parse_size(""), None);
        assert_eq!(Config::parse_size("0"), None);
        assert_eq!(Config::parse_size("0K"), None);
        assert_eq!(Config::parse_size("K"), None);
        assert_eq!(Config::parse_size("abc"), None);
        assert_eq!(Config::parse_size("-1"), None);
    }
}
