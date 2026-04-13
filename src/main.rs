use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use servers_sidecar::{ServerConfig, HistorySample, load_server_config, update_history, ServerStatusRow, derive_status, uptime_pct, ServersPayload};
use vzglyd_sidecar::{Error, https_get, poll_loop, split_https_url, tcp_connect};

const CONFIG_JSON: &str = include_str!("../config/servers.json");

thread_local! {
    static HISTORY: RefCell<HashMap<String, Vec<HistorySample>>> =
        RefCell::new(HashMap::new());
}

fn fetch() -> Result<Vec<u8>, Error> {
    let now_secs = now_unix_secs();
    let configs = load_server_config(CONFIG_JSON);
    let mut rows = Vec::with_capacity(configs.len());

    HISTORY.with(|history_cell| {
        let mut history = history_cell.borrow_mut();
        for config in &configs {
            let (ok, response_ms) = match config.check_type.as_str() {
                "http" => check_http(config),
                "tcp" => check_tcp(config),
                _ => (false, 0),
            };

            let entry = history.entry(config.name.clone()).or_default();
            update_history(entry, now_secs, ok);
            rows.push(ServerStatusRow {
                name: config.name.clone(),
                region: config.region.clone(),
                check_type: config.check_type.clone(),
                status: derive_status(ok, response_ms, entry),
                uptime: uptime_pct(entry),
                response_ms: response_ms.to_string(),
            });
        }
    });

    let payload = ServersPayload {
        updated: format!("Updated {}", utc_hhmm(now_secs)),
        rows,
    };
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn check_http(config: &ServerConfig) -> (bool, u32) {
    let Some(url) = config.url.as_deref() else {
        return (false, 0);
    };
    let Ok((host, path)) = split_https_url(url) else {
        return (false, 0);
    };
    let started = Instant::now();
    let ok = https_get(&host, &path).is_ok();
    (ok, started.elapsed().as_millis() as u32)
}

fn check_tcp(config: &ServerConfig) -> (bool, u32) {
    let Some(host) = config.host.as_deref() else {
        return (false, 0);
    };
    match tcp_connect(host, config.port, config.timeout_ms) {
        Ok(duration) => (true, duration.as_millis() as u32),
        Err(_) => (false, 0),
    }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn utc_hhmm(epoch_secs: u64) -> String {
    let seconds_today = epoch_secs % 86_400;
    let hours = seconds_today / 3_600;
    let minutes = (seconds_today / 60) % 60;
    format!("{hours:02}:{minutes:02}")
}

#[cfg(target_arch = "wasm32")]
fn main() {
    poll_loop(30, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("servers-sidecar is intended for wasm32-wasip1");
}
