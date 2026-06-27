use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub port: u16,
    pub public_base_url: String,
    pub database_url: String,
    pub admin_path: String,
    pub admin_bootstrap_user: String,
    pub admin_bootstrap_password: String,
    pub session_ttl_min: i64,
    pub public_dir: String,
    pub backup_dir: String,
    pub json_body_limit_bytes: usize,
    pub admin_ip_allowlist: Vec<String>,
    pub bot_auto_tick_enabled: bool,
    pub bot_auto_tick_interval_seconds: u64,
    pub bot_auto_tick_limit: i64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            app_env: read("APP_ENV", "development"),
            port: read("PORT", "3000").parse().unwrap_or(3000),
            public_base_url: read("PUBLIC_BASE_URL", "http://localhost:3000"),
            database_url: read("DATABASE_URL", "postgres://cq:cq_password@localhost:5432/cq"),
            admin_path: read("ADMIN_PATH", "admin"),
            admin_bootstrap_user: read("ADMIN_BOOTSTRAP_USER", "admin"),
            admin_bootstrap_password: read("ADMIN_BOOTSTRAP_PASSWORD", "change_me"),
            session_ttl_min: read_positive_i64("SESSION_TTL_MIN", 1440),
            public_dir: read("PUBLIC_DIR", "web/player/dist"),
            backup_dir: read("BACKUP_DIR", "data/backup"),
            json_body_limit_bytes: read("JSON_BODY_LIMIT_BYTES", "262144")
                .parse()
                .unwrap_or(262_144),
            admin_ip_allowlist: read("ADMIN_IP_ALLOWLIST", "")
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            bot_auto_tick_enabled: read_bool("BOT_AUTO_TICK_ENABLED", true),
            bot_auto_tick_interval_seconds: read_positive_u64("BOT_AUTO_TICK_INTERVAL_SECONDS", 30),
            bot_auto_tick_limit: read_positive_i64("BOT_AUTO_TICK_LIMIT", 50),
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.port))
    }
}

fn read(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn read_positive_i64(name: &str, default: i64) -> i64 {
    match env::var(name).ok().and_then(|value| value.parse::<i64>().ok()) {
        Some(value) if value > 0 => value,
        Some(value) => {
            tracing::warn!(name, value, default, "invalid non-positive config value, using default");
            default
        }
        None => default,
    }
}

fn read_positive_u64(name: &str, default: u64) -> u64 {
    match env::var(name).ok().and_then(|value| value.parse::<u64>().ok()) {
        Some(value) if value > 0 => value,
        Some(value) => {
            tracing::warn!(name, value, default, "invalid non-positive config value, using default");
            default
        }
        None => default,
    }
}

fn read_bool(name: &str, default: bool) -> bool {
    match env::var(name).ok().map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes" | "on") => true,
        Some(value) if matches!(value.as_str(), "0" | "false" | "no" | "off") => false,
        Some(value) => {
            tracing::warn!(name, value, default, "invalid bool config value, using default");
            default
        }
        None => default,
    }
}
