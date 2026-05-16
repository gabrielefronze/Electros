use tauri::{AppHandle, Manager, WebviewWindow};

const TITLEBAR_BOOTSTRAP: &str = include_str!("../resources/titlebar-bootstrap.js");

fn asset_bases() -> (&'static str, &'static str) {
    if cfg!(debug_assertions) {
        (
            "http://localhost:5173/titlebar",
            "http://localhost:5173/electros",
        )
    } else {
        ("../titlebar", ".")
    }
}

pub fn bases_init_script() -> String {
    let (titlebar, gui) = asset_bases();
    format!(
        "window.__ELECTROS_TITLEBAR_BASE__ = {titlebar:?}; window.__ELECTROS_GUI_BASE__ = {gui:?};"
    )
}

pub fn bootstrap_script() -> String {
    format!("{}\n{}", bases_init_script(), TITLEBAR_BOOTSTRAP)
}

pub fn inject_into_window(window: &WebviewWindow) {
    let script = bootstrap_script();
    let _ = window.eval(&script);
}

pub fn should_install_titlebar(label: &str) -> bool {
    label == "main"
        || label.starts_with("ssh-")
        || label.starts_with("rdp-")
        || label.starts_with("popup-")
}

pub fn inject_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        inject_into_window(&window);
    }
}
