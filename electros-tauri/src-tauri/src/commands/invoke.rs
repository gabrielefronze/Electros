use crate::commands::{backgrounds, config, daemons, remotes, system, window};
use crate::safe_storage;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Deserialize)]
pub struct InvokeRequest {
    pub channel: String,
    pub args: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct InvokeError {
    pub error: String,
}

pub async fn dispatch(
    app: AppHandle,
    state: Arc<AppState>,
    req: InvokeRequest,
) -> Result<Value, String> {
    let args = req.args.unwrap_or(Value::Null);
    let arr = args.as_array().cloned().unwrap_or_default();

    match req.channel.as_str() {
        "read-config" => Ok(config::read_config().await),
        "write-config" => {
            let cfg = arr.first().cloned().unwrap_or(args);
            Ok(config::write_config(cfg).await)
        }
        "read-hosts" => Ok(config::read_hosts().await),
        "write-hosts" => {
            let hosts = arr.first().cloned().unwrap_or(args);
            Ok(config::write_hosts(hosts).await)
        }
        "cors-safe-fetch" => {
            let url = arr
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(system::cors_safe_fetch(url).await)
        }
        "list-backgrounds" => Ok(backgrounds::list_backgrounds().await),
        "get-background-data" => {
            let path = arr
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let thumb = arr.get(1).and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(backgrounds::get_background_data(path, thumb).await)
        }
        "import-background" => {
            let app_dialog = app.clone();
            let path = tokio::task::spawn_blocking(move || {
                app_dialog
                    .dialog()
                    .file()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                    .blocking_pick_file()
            })
            .await
            .map_err(|e| e.to_string())?;
            match path {
                Some(p) => {
                    let path_str = p
                        .into_path()
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .to_string();
                    Ok(backgrounds::import_background_from_path(path_str).await)
                }
                None => Ok(serde_json::json!({ "success": false, "canceled": true })),
            }
        }
        "save-background-from-url" => {
            let url = arr
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let filename = arr.get(1).and_then(|v| v.as_str()).map(String::from);
            let subfolder = arr.get(2).and_then(|v| v.as_str()).map(String::from);
            Ok(backgrounds::save_background_from_url(url, filename, subfolder).await)
        }
        "delete-background" => {
            let path = arr
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(backgrounds::delete_background(path).await)
        }
        "convert-existing-backgrounds" => Ok(backgrounds::convert_existing_backgrounds().await),
        "minimize-window" => {
            window::minimize(&app)?;
            Ok(Value::Null)
        }
        "maximize-window" => {
            window::maximize(&app)?;
            Ok(Value::Null)
        }
        "close-window" => {
            window::close(&app)?;
            Ok(Value::Null)
        }
        "toggle-full-screen" => {
            window::toggle_fullscreen(&app)?;
            Ok(Value::Null)
        }
        "check-port" => {
            let obj = arr.first().cloned().unwrap_or(args);
            let ip = obj
                .get("ip")
                .and_then(|v| v.as_str())
                .unwrap_or("127.0.0.1")
                .to_string();
            let port = obj.get("port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            Ok(system::check_port(ip, port).await)
        }
        "os-prefers-dark-theme" => Ok(system::os_prefers_dark_theme()),
        "os-prefers-reduced-transparency" => Ok(system::os_prefers_reduced_transparency()),
        "open-browser" => {
            let obj = arr.first().cloned().unwrap_or(args);
            Ok(system::open_browser(obj).await)
        }
        "open-dot-config" => {
            let obj = arr.first().cloned().unwrap_or(args);
            Ok(system::open_dot_config(obj).await)
        }
        "app-version" => Ok(system::app_version()),
        "safestorage-encrypt" => {
            let obj = arr.first().cloned().unwrap_or(args);
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let refuse = obj
                .get("refuseUnsafe")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            safe_storage::encrypt_string(value, refuse)
        }
        "safestorage-decrypt" => {
            let obj = arr.first().cloned().unwrap_or(args);
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            safe_storage::decrypt_string(value)
        }
        "get-daemons-log" => Ok(daemons::get_daemons_log(&state)),
        "open-ssh" => {
            let details = arr.first().cloned().unwrap_or(args);
            remotes::open_ssh(app.clone(), state, details).await
        }
        "open-rdp" => {
            let details = arr.first().cloned().unwrap_or(args);
            remotes::open_rdp(app.clone(), state, details).await
        }
        "launch-rdp-process" => {
            let payload = arr.first().cloned().unwrap_or(args);
            let label = state
                .last_rdp_window
                .lock()
                .clone()
                .or_else(|| {
                    app.webview_windows()
                        .keys()
                        .find(|k| k.starts_with("rdp-"))
                        .cloned()
                })
                .unwrap_or_else(|| "main".into());
            remotes::launch_rdp_process(app.clone(), state, label, payload).await
        }
        "cleanup-rdp-process" => {
            let port = arr.first().and_then(|v| v.as_u64()).map(|p| p as u16);
            let label = state
                .last_rdp_window
                .lock()
                .clone()
                .or_else(|| {
                    app.webview_windows()
                        .keys()
                        .find(|k| k.starts_with("rdp-"))
                        .cloned()
                })
                .unwrap_or_else(|| "main".into());
            remotes::cleanup_rdp_process(state, label, port).await
        }
        other => Err(format!("Unknown IPC channel: {other}")),
    }
}

pub async fn electron_send(app: AppHandle, channel: String, args: Option<Value>) -> Result<(), String> {
    let _ = (app, args);
    if channel == "open-rdp" {
        // no-op; parity with electron send
    }
    Ok(())
}

pub fn emit_deep_link(app: &AppHandle, url: String) {
    let _ = app.emit("deep-link", url);
}
