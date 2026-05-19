use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub port: u16,
    pub broker_host: String,
    pub broker_port: u16,
    pub broker_username: Option<String>,
    pub broker_password: Option<String>,
    pub broker_ssl: bool,
    pub bots_path: PathBuf,
    pub pool_bots: Vec<String>,
    pub command_timeout_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub r2_enabled: bool,
    pub r2_bucket: String,
    pub r2_endpoint_url: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub r2_prefix: String,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let dotenv = Dotenv::load(".env")?;
        Ok(Self {
            port: dotenv.parse("RS_ORCHESTRATOR_PORT", 8001),
            broker_host: dotenv
                .get("BROKER_HOST")
                .unwrap_or_else(|| "localhost".to_string()),
            broker_port: dotenv.parse("BROKER_PORT", 1883),
            broker_username: empty_to_none(dotenv.get("BROKER_USERNAME")),
            broker_password: empty_to_none(dotenv.get("BROKER_PASSWORD")),
            broker_ssl: dotenv.parse_bool("BROKER_SSL", false),
            bots_path: PathBuf::from(dotenv.get("BOTS_PATH").unwrap_or_else(|| ".".to_string())),
            pool_bots: parse_pool_bots(&dotenv),
            command_timeout_secs: dotenv.parse("COMMAND_TIMEOUT_SECS", 30),
            heartbeat_timeout_secs: dotenv.parse("HEARTBEAT_TIMEOUT_SECS", 30),
            r2_enabled: dotenv.parse_bool("R2_ENABLED", false),
            r2_bucket: dotenv.get("R2_BUCKET").unwrap_or_default(),
            r2_endpoint_url: dotenv.get("R2_ENDPOINT_URL").unwrap_or_default(),
            r2_access_key_id: dotenv.get("R2_ACCESS_KEY_ID").unwrap_or_default(),
            r2_secret_access_key: dotenv.get("R2_SECRET_ACCESS_KEY").unwrap_or_default(),
            r2_prefix: dotenv
                .get("R2_PREFIX")
                .unwrap_or_else(|| "bots".to_string()),
        })
    }

    pub fn command_timeout(&self) -> Duration {
        Duration::from_secs(self.command_timeout_secs)
    }

    pub fn heartbeat_timeout(&self) -> Duration {
        Duration::from_secs(self.heartbeat_timeout_secs)
    }
}

fn parse_pool_bots(dotenv: &Dotenv) -> Vec<String> {
    dotenv
        .get("POOL_BOTS")
        .unwrap_or_else(|| "warmbot_1,warmbot_2,warmbot_3".to_string())
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        if item.trim().is_empty() {
            None
        } else {
            Some(item)
        }
    })
}

#[derive(Debug, Default)]
struct Dotenv {
    values: HashMap<String, String>,
}

impl Dotenv {
    fn load(path: &str) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                tracing::info!(path = %path, "loaded environment file");
                Ok(Self {
                    values: parse_dotenv(&content),
                })
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!(
                    path = %path,
                    "environment file not found; relying on system environment variables"
                );
                Ok(Self::default())
            }
            Err(err) => Err(err.into()),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key)
            .ok()
            .map(|val| unquote_dotenv_value(val.trim()))
            .or_else(|| self.values.get(key).cloned())
    }

    fn parse<T>(&self, key: &str, default: T) -> T
    where
        T: std::str::FromStr,
    {
        self.get(key)
            .and_then(|value| value.parse::<T>().ok())
            .unwrap_or(default)
    }

    fn parse_bool(&self, key: &str, default: bool) -> bool {
        self.get(key)
            .and_then(|val| {
                let s = val.trim().to_lowercase();
                if s == "true" || s == "1" || s == "yes" || s == "on" {
                    Some(true)
                } else if s == "false" || s == "0" || s == "no" || s == "off" {
                    Some(false)
                } else {
                    None
                }
            })
            .unwrap_or(default)
    }
}

fn parse_dotenv(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(parse_dotenv_line)
        .collect::<HashMap<_, _>>()
}

fn parse_dotenv_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let line = line.strip_prefix("export ").unwrap_or(line);
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    Some((key.to_string(), unquote_dotenv_value(value.trim())))
}

fn unquote_dotenv_value(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let quote = bytes[0];
        if (quote == b'\'' || quote == b'"') && bytes[value.len() - 1] == quote {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::parse_dotenv;

    #[test]
    fn parses_dotenv_values() {
        let values = parse_dotenv(
            r#"
            # ignored
            R2_ENABLED=true
            export BROKER_HOST=host.docker.internal
            EMPTY=
            QUOTED="bots"
            SINGLE='warmbot_1,warmbot_2'
            "#,
        );

        assert_eq!(values.get("R2_ENABLED").map(String::as_str), Some("true"));
        assert_eq!(
            values.get("BROKER_HOST").map(String::as_str),
            Some("host.docker.internal")
        );
        assert_eq!(values.get("EMPTY").map(String::as_str), Some(""));
        assert_eq!(values.get("QUOTED").map(String::as_str), Some("bots"));
        assert_eq!(
            values.get("SINGLE").map(String::as_str),
            Some("warmbot_1,warmbot_2")
        );
    }
}
