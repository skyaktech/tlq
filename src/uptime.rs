use std::sync::OnceLock;
use std::time::{Duration, Instant};

static START: OnceLock<Instant> = OnceLock::new();

/// Capture the process start time. Call once early in `main`.
pub fn init() {
    START.get_or_init(Instant::now);
}

/// Time elapsed since `init` (or first access if `init` was not called).
pub fn uptime() -> Duration {
    START.get_or_init(Instant::now).elapsed()
}

/// Uptime as a compact human-readable string, e.g. `1d 2h 3m 4s` or `5s`.
pub fn uptime_human() -> String {
    format_duration(uptime())
}

fn format_duration(d: Duration) -> String {
    let total = d.as_secs();
    let units = [
        ('d', total / 86_400),
        ('h', (total % 86_400) / 3_600),
        ('m', (total % 3_600) / 60),
        ('s', total % 60),
    ];

    units
        .iter()
        // Drop leading zero units, but always keep seconds as the fallback.
        .skip_while(|&&(unit, value)| value == 0 && unit != 's')
        .map(|(unit, value)| format!("{value}{unit}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_seconds_only() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(5)), "5s");
        assert_eq!(format_duration(Duration::from_secs(59)), "59s");
    }

    #[test]
    fn formats_minutes_and_hours() {
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m 0s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn formats_days() {
        assert_eq!(format_duration(Duration::from_secs(90_061)), "1d 1h 1m 1s");
        assert_eq!(format_duration(Duration::from_secs(86_400)), "1d 0h 0m 0s");
    }

    #[test]
    fn ignores_sub_second_remainder() {
        assert_eq!(format_duration(Duration::from_millis(1500)), "1s");
    }
}
