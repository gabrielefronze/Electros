use crate::state::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tauri::Manager;

pub fn resource_base(_app: &tauri::AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        // electros-tauri/ when running cargo tauri dev
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.parent().unwrap_or(&manifest).to_path_buf();
    }
    _app.path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn daemons_dir(app: &tauri::AppHandle) -> PathBuf {
    let base = resource_base(app);
    if cfg!(debug_assertions) {
        return base.join("electros-daemons");
    }
    base.join("electros-daemons")
}

fn platform_subdir() -> (String, String) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let os_dir = match os {
        "macos" => "mac".to_string(),
        "windows" => "win".to_string(),
        _ => "linux".to_string(),
    };
    let arch_dir = if arch == "aarch64" {
        "arm64".to_string()
    } else if arch == "x86_64" {
        "x64".to_string()
    } else {
        arch.to_string()
    };
    (os_dir, arch_dir)
}

pub fn resolve_daemon_binary(app: &tauri::AppHandle) -> Option<PathBuf> {
    let (os_dir, arch) = platform_subdir();
    let dir = daemons_dir(app).join(&os_dir).join(&arch);
    if cfg!(target_os = "macos") {
        for name in [
            "elemento_client_daemons.app/Contents/MacOS/elemento_client_daemons",
            "elemento_client_daemons.app/Contents/MacOS/daemon_launcher",
        ] {
            let p = dir.join(name);
            if p.exists() {
                return Some(p);
            }
        }
    } else if cfg!(target_os = "linux") {
        let name = if arch == "aarch64" {
            "elemento_daemons_linux_arm"
        } else {
            "elemento_daemons_linux_x86"
        };
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    } else if cfg!(target_os = "windows") {
        for name in [
            "elemento_daemons_win_x64.exe",
            "elemento_daemons_win_x86.exe",
            "elemento_daemons_windows_x64.exe",
        ] {
            let p = dir.join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

pub fn launch_daemons(app: &tauri::AppHandle, state: &Arc<AppState>) {
    if state.no_daemons {
        state.push_log("[INFO] Elemento Client Daemons disabled (--no-daemons).\n");
        return;
    }
    let Some(exec) = resolve_daemon_binary(app) else {
        state.push_log("[WARN] Daemon binary not found. Run electros-tauri/populate-daemons.sh\n");
        return;
    };
    let mut child = match Command::new(&exec)
        .env("GUI_APP", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            state.push_log(format!("[ERROR] Failed to spawn daemons: {e}\n"));
            return;
        }
    };

    if let Some(stdout) = child.stdout.take() {
        let state_c = state.clone();
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            let mut handle = stdout;
            loop {
                match handle.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let s = String::from_utf8_lossy(&buf[..n]).to_string();
                        state_c.push_log(s);
                    }
                    Err(_) => break,
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let state_c = state.clone();
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            let mut handle = stderr;
            loop {
                match handle.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let s = String::from_utf8_lossy(&buf[..n]).to_string();
                        state_c.push_log(s);
                    }
                    Err(_) => break,
                }
            }
        });
    }

    *state.daemon_child.lock() = Some(child);
    state.push_log("[INFO] Elemento Client Daemons started.\n");
}

pub fn get_daemons_log(state: &Arc<AppState>) -> Value {
    let buf = state.daemon_logs.lock();
    json!(buf.clone())
}

pub fn terminate_daemons(state: &Arc<AppState>) {
    if let Some(mut child) = state.daemon_child.lock().take() {
        let _ = child.kill();
    }
}
