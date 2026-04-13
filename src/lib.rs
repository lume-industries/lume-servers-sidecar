use serde::{Deserialize, Serialize};

pub const HISTORY_WINDOW_SECS: u64 = 86_400;
pub const WARNING_MS: u32 = 1_000;
pub const DEGRADED_MS: u32 = 3_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
    pub name: String,
    pub region: String,
    pub check_type: String,
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    pub timeout_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistorySample {
    pub timestamp: u64,
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerStatusRow {
    pub name: String,
    pub region: String,
    pub check_type: String,
    pub status: String,
    pub uptime: String,
    pub response_ms: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServersPayload {
    pub updated: String,
    pub rows: Vec<ServerStatusRow>,
}

#[derive(Deserialize)]
struct ConfigFile {
    servers: Vec<ServerConfigEntry>,
}

#[derive(Deserialize)]
struct ServerConfigEntry {
    name: String,
    region: String,
    check_type: String,
    url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    timeout_ms: Option<u32>,
}

pub fn load_server_config(json: &str) -> Vec<ServerConfig> {
    let file: ConfigFile =
        serde_json::from_str(json).expect("decode servers.json");
    file.servers
        .into_iter()
        .map(|entry| ServerConfig {
            name: entry.name,
            region: entry.region,
            check_type: entry.check_type,
            url: entry.url,
            host: entry.host,
            port: entry.port.unwrap_or(80),
            timeout_ms: entry.timeout_ms.unwrap_or(5_000),
        })
        .collect()
}

pub fn update_history(entries: &mut Vec<HistorySample>, timestamp: u64, ok: bool) {
    entries.push(HistorySample { timestamp, ok });
    let cutoff = timestamp.saturating_sub(HISTORY_WINDOW_SECS);
    entries.retain(|entry| entry.timestamp >= cutoff);
}

pub fn uptime_pct(entries: &[HistorySample]) -> String {
    if entries.is_empty() {
        return "--".to_string();
    }
    let ok = entries.iter().filter(|entry| entry.ok).count() as f32;
    format!("{:.2}%", ok / entries.len() as f32 * 100.0)
}

pub fn derive_status(ok: bool, response_ms: u32, entries: &[HistorySample]) -> String {
    if !ok {
        let trailing_failures =
            entries.iter().rev().take(3).all(|entry| !entry.ok) && entries.len() >= 3;
        return if trailing_failures {
            "down".to_string()
        } else {
            "degraded".to_string()
        };
    }
    if response_ms >= DEGRADED_MS {
        "degraded".to_string()
    } else if response_ms >= WARNING_MS {
        "warning".to_string()
    } else {
        "healthy".to_string()
    }
}
