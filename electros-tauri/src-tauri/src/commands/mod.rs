pub mod backgrounds;
pub mod daemons;
pub mod invoke;
pub mod remotes;

pub mod config {
    use crate::paths::{config_path, ensure_elemento_dirs, hosts_path};
    use serde_json::{json, Value};
    use std::fs;

    pub async fn read_config() -> Value {
        ensure_elemento_dirs();
        if let Ok(data) = fs::read_to_string(config_path()) {
            if let Ok(v) = serde_json::from_str(&data) {
                return v;
            }
        }
        json!({})
    }

    pub async fn write_config(mut config: Value) -> Value {
        ensure_elemento_dirs();
        if config.get("config").is_some() {
            config = config.get("config").cloned().unwrap_or(json!({}));
        }
        let mut existing = read_config().await;
        if let (Some(existing_obj), Some(new_obj)) =
            (existing.as_object_mut(), config.as_object())
        {
            for (k, v) in new_obj {
                existing_obj.insert(k.clone(), v.clone());
            }
        } else {
            existing = config;
        }
        match fs::write(
            config_path(),
            serde_json::to_string_pretty(&existing).unwrap_or_else(|_| "{}".into()),
        ) {
            Ok(()) => json!(true),
            Err(e) => {
                eprintln!("Error writing config: {e}");
                json!(false)
            }
        }
    }

    pub async fn read_hosts() -> Value {
        ensure_elemento_dirs();
        if let Ok(data) = fs::read_to_string(hosts_path()) {
            let lines: Vec<&str> = data.lines().filter(|l| !l.trim().is_empty()).collect();
            return json!(lines);
        }
        json!([])
    }

    pub async fn write_hosts(hosts: Value) -> Value {
        ensure_elemento_dirs();
        let lines: Vec<String> = if let Some(arr) = hosts.as_array() {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            vec![]
        };
        match fs::write(hosts_path(), lines.join("\n")) {
            Ok(()) => json!(true),
            Err(e) => {
                eprintln!("Error writing hosts: {e}");
                json!(false)
            }
        }
    }
}

pub mod system {
    use serde_json::{json, Value};
    use std::time::Duration;

    const POTD_HOSTS: &[&str] = &[
        "commons.wikimedia.org",
        "upload.wikimedia.org",
        "peapix.com",
        "img.peapix.com",
        "www.bing.com",
        "cn.bing.com",
        "www2.bing.com",
        "api.nasa.gov",
        "apod.nasa.gov",
    ];

    pub async fn cors_safe_fetch(url_string: String) -> Value {
        let parsed = match url::Url::parse(&url_string) {
            Ok(u) => u,
            Err(_) => {
                return json!({
                    "ok": false, "status": 0, "text": "", "contentType": "",
                    "error": "Invalid URL"
                });
            }
        };
        if parsed.scheme() != "https" && parsed.scheme() != "http" {
            return json!({
                "ok": false, "status": 0, "text": "", "contentType": "",
                "error": "Invalid protocol"
            });
        }
        if !POTD_HOSTS.contains(&parsed.host_str().unwrap_or("")) {
            return json!({
                "ok": false, "status": 0, "text": "", "contentType": "",
                "error": format!("Host not allowed: {}", parsed.host_str().unwrap_or(""))
            });
        }
        let client = reqwest::Client::builder()
            .user_agent("Electros/3.1 (https://elemento.cloud/electros; hello@elemento.cloud)")
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        match client.get(url_string).send().await {
            Ok(res) => {
                let status = res.status().as_u16();
                let content_type = res
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                let text = res.text().await.unwrap_or_default();
                json!({
                    "ok": status >= 200 && status < 300,
                    "status": status,
                    "text": text,
                    "contentType": content_type,
                })
            }
            Err(e) => json!({
                "ok": false, "status": 0, "text": "", "contentType": "",
                "error": e.to_string()
            }),
        }
    }

    pub async fn check_port(ip: String, port: u16) -> Value {
        let addr = format!("{ip}:{port}");
        match tokio::time::timeout(
            Duration::from_secs(1),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => json!(true),
            _ => json!(false),
        }
    }

    pub fn os_prefers_dark_theme() -> Value {
        let dark = dark_light::detect() == dark_light::Mode::Dark;
        json!(dark)
    }

    pub fn os_prefers_reduced_transparency() -> Value {
        json!(false)
    }

    pub async fn open_browser(url: Value) -> Value {
        let u = url
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if u.is_empty() {
            return json!(null);
        }
        let _ = open::that(&u);
        json!(null)
    }

    pub async fn open_dot_config(file: Value) -> Value {
        let path = file.get("file").and_then(|v| v.as_str());
        let mut target = crate::paths::elemento_dir();
        if let Some(f) = path {
            for part in f.split('/') {
                if !part.is_empty() {
                    target = target.join(part);
                }
            }
        }
        let _ = open::that(&target);
        json!(null)
    }

    pub fn app_version() -> Value {
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "node": { "tauri": true }
        })
    }
}

pub mod window {
    use tauri::{AppHandle, Manager};

    pub fn minimize(app: &AppHandle) -> Result<(), String> {
        app.get_webview_window("main")
            .ok_or("main window not found")?
            .minimize()
            .map_err(|e| e.to_string())
    }
    pub fn maximize(app: &AppHandle) -> Result<(), String> {
        let w = app.get_webview_window("main").ok_or("main window not found")?;
        if w.is_maximized().map_err(|e| e.to_string())? {
            w.unmaximize().map_err(|e| e.to_string())?;
        } else {
            w.maximize().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn close(app: &AppHandle) -> Result<(), String> {
        app.get_webview_window("main")
            .ok_or("main window not found")?
            .close()
            .map_err(|e| e.to_string())
    }
    pub fn toggle_fullscreen(app: &AppHandle) -> Result<(), String> {
        let w = app.get_webview_window("main").ok_or("main window not found")?;
        let fs = w.is_fullscreen().map_err(|e| e.to_string())?;
        w.set_fullscreen(!fs).map_err(|e| e.to_string())
    }
}
