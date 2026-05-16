use crate::commands::daemons::resource_base;
use crate::state::{pick_port, release_port, AppState};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const ELECTRON_SHIM: &str = include_str!("../../resources/electron-shim.js");

fn inject_shim(window: &WebviewWindow) {
    let _ = window.eval(ELECTRON_SHIM);
    crate::titlebar::inject_into_window(window);
}

fn remotes_base(app: &AppHandle) -> PathBuf {
    let base = resource_base(app);
    if cfg!(debug_assertions) {
        return base.join("electros").join("remotes");
    }
    base.join("electros").join("remotes")
}

fn mstsc_path(app: &AppHandle) -> PathBuf {
    remotes_base(app).join("rdp").join(if cfg!(windows) {
        "mstsc-rs.exe"
    } else {
        "mstsc-rs"
    })
}

fn ssh_script_path(app: &AppHandle) -> PathBuf {
    remotes_base(app).join("ssh").join("ssh.cjs")
}

pub async fn open_ssh(
    app: AppHandle,
    state: Arc<AppState>,
    connection: Value,
) -> Result<Value, String> {
    let vm_name = connection
        .get("vmName")
        .and_then(|v| v.as_str())
        .unwrap_or("SSH");
    let ip = connection.get("ip").and_then(|v| v.as_str()).unwrap_or("");
    let username = connection
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let password = connection
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let port = pick_port(&state);
    let ssh_cjs = ssh_script_path(&app);
    if !ssh_cjs.exists() {
        release_port(&state, port);
        return Err(format!(
            "SSH server script not found at {}. Stage remotes via populate-daemons.",
            ssh_cjs.display()
        ));
    }

    let base_dir = resource_base(&app);
    let port_str = port.to_string();
    let script = format!(
        "const m = require({}); m.runSSHServer({}, {}).catch(console.error);",
        serde_json::to_string(ssh_cjs.to_string_lossy().as_ref()).unwrap_or_default(),
        port_str,
        serde_json::to_string(base_dir.to_string_lossy().as_ref()).unwrap_or_default()
    );
    let child = Arc::new(Mutex::new(
        Command::new("node")
            .args(["-e", &script])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start SSH server (is node installed?): {e}"))?,
    ));

    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    let label = format!("ssh-{}", port);
    let url = format!(
        "http://127.0.0.1:{port}/?host={}&username={}&password={}",
        urlencoding_query(ip),
        urlencoding_query(username),
        urlencoding_query(password)
    );

    let window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(url.parse().unwrap()))
        .title(format!("SSH connection to {vm_name}"))
        .inner_size(1024.0, 768.0)
        .decorations(false)
        .zoom_hotkeys_enabled(true)
        .build()
        .map_err(|e| e.to_string())?;
    inject_shim(&window);

    let app_c = app.clone();
    let state_c = state.clone();
    let label_c = label.clone();
    let child_c = child.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            let _ = app_c.emit_to(&label_c, "window-close", ());
            release_port(&state_c, port);
            if let Ok(mut c) = child_c.lock() {
                let _ = c.kill();
            }
        }
    });

    Ok(json!(window.label()))
}

fn urlencoding_query(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

pub async fn open_rdp(
    app: AppHandle,
    state: Arc<AppState>,
    connection: Value,
) -> Result<Value, String> {
    let vm_name = connection
        .get("vmName")
        .and_then(|v| v.as_str())
        .unwrap_or("RDP");
    let _ip = connection.get("ip").and_then(|v| v.as_str()).unwrap_or("");

    let label = format!("rdp-{}", rand::random::<u32>());
    let url = if cfg!(debug_assertions) {
        WebviewUrl::External(
            "http://localhost:5173/electros/remotes/rdp/rdp.html"
                .parse()
                .unwrap(),
        )
    } else {
        WebviewUrl::App("remotes/rdp/rdp.html".into())
    };

    let window = WebviewWindowBuilder::new(&app, &label, url)
        .title(format!("RDP - {vm_name}"))
        .inner_size(1280.0, 800.0)
        .decorations(false)
        .zoom_hotkeys_enabled(true)
        .build()
        .map_err(|e| e.to_string())?;
    inject_shim(&window);

    let label_str = window.label().to_string();
    *state.last_rdp_window.lock() = Some(label_str.clone());
    Ok(json!(label_str))
}

pub async fn launch_rdp_process(
    app: AppHandle,
    state: Arc<AppState>,
    window_label: String,
    payload: Value,
) -> Result<Value, String> {
    let credentials = payload.get("credentials").cloned().unwrap_or(json!({}));
    let width = payload
        .get("width")
        .and_then(|v| v.as_u64())
        .unwrap_or(1280) as u32;
    let height = payload
        .get("height")
        .and_then(|v| v.as_u64())
        .unwrap_or(800) as u32;

    let ip = credentials.get("ip").and_then(|v| v.as_str()).unwrap_or("");
    let username = credentials
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let domain = credentials
        .get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let password = credentials
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let ws_port = pick_port(&state);
    let mstsc = mstsc_path(&app);
    if !mstsc.exists() {
        release_port(&state, ws_port);
        return Err(format!("mstsc-rs not found at {}", mstsc.display()));
    }

    let child = Command::new(&mstsc)
        .args([
            "--target",
            ip,
            "--user",
            username,
            "--dom",
            domain,
            "--pass",
            password,
            "--width",
            &width.to_string(),
            "--height",
            &height.to_string(),
            "--ws_port",
            &ws_port.to_string(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    state
        .rdp_processes
        .lock()
        .insert(window_label.clone(), child);
    state.rdp_ports.lock().insert(window_label.clone(), ws_port);

    let app_c = app.clone();
    let state_c = state.clone();
    let wl = window_label.clone();
    std::thread::spawn(move || {
        if let Some(mut proc) = state_c.rdp_processes.lock().remove(&wl) {
            let _ = proc.wait();
        }
        if let Some(port) = state_c.rdp_ports.lock().remove(&wl) {
            release_port(&state_c, port);
        }
        let _ = app_c.emit_to(&wl, "rdp-process-closed", ());
    });

    Ok(json!({ "success": true, "ws_port": ws_port }))
}

pub async fn cleanup_rdp_process(
    state: Arc<AppState>,
    window_label: String,
    port: Option<u16>,
) -> Result<Value, String> {
    if let Some(p) = port {
        release_port(&state, p);
    }
    if let Some(mut child) = state.rdp_processes.lock().remove(&window_label) {
        let _ = child.kill();
    }
    if let Some(p) = state.rdp_ports.lock().remove(&window_label) {
        release_port(&state, p);
    }
    Ok(json!({ "success": true }))
}

pub async fn create_popup(app: AppHandle, options: Value) -> Result<Value, String> {
    let url = options
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("URL is required for popup window")?;
    let width = options.get("width").and_then(|v| v.as_f64()).unwrap_or(800.0);
    let height = options.get("height").and_then(|v| v.as_f64()).unwrap_or(600.0);
    let title = options
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Electros");
    let is_file = options.get("isFile").and_then(|v| v.as_bool()).unwrap_or(false);

    let label = format!("popup-{}", rand::random::<u32>());
    let webview_url = if is_file {
        WebviewUrl::App(url.into())
    } else {
        WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?)
    };

    let decorations = !options
        .get("defaultTitlebar")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let window = WebviewWindowBuilder::new(&app, &label, webview_url)
        .title(title)
        .inner_size(width, height)
        .decorations(decorations)
        .zoom_hotkeys_enabled(true)
        .build()
        .map_err(|e| e.to_string())?;
    inject_shim(&window);

    Ok(json!(window.label()))
}

pub fn remotes_resource_hint(app: &AppHandle) -> PathBuf {
    resource_base(app).join("electros").join("remotes")
}
