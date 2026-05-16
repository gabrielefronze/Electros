mod commands;
mod paths;
mod safe_storage;
mod state;
mod titlebar;

use commands::backgrounds;
use commands::daemons::{self, launch_daemons, terminate_daemons};
use commands::invoke::{self, InvokeRequest};
use state::AppState;
use std::sync::Arc;
use tauri::webview::PageLoadEvent;
use tauri::Manager;

const ELECTRON_SHIM: &str = include_str!("../resources/electron-shim.js");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let no_daemons = std::env::args().any(|a| a == "--no-daemons");
    let app_state = AppState::new(no_daemons);
    let app_state_on_exit = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .append_invoke_initialization_script(titlebar::bases_init_script())
        .append_invoke_initialization_script(ELECTRON_SHIM)
        .append_invoke_initialization_script(include_str!(
            "../resources/titlebar-bootstrap.js"
        ))
        .on_page_load(|webview, payload| {
            if payload.event() != PageLoadEvent::Finished {
                return;
            }
            let label = webview.label();
            if !titlebar::should_install_titlebar(label) {
                return;
            }
            if let Some(window) = webview.app_handle().get_webview_window(label) {
                titlebar::inject_into_window(&window);
            }
        })
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            electron_invoke,
            create_popup,
            electron_send,
        ])
        .register_uri_scheme_protocol("elemento-bg", move |_app, request| {
            match backgrounds::serve_elemento_bg(&request.uri().to_string()) {
                Ok(resp) => resp,
                Err(msg) => tauri::http::Response::builder()
                    .status(404)
                    .body(msg.into_bytes())
                    .unwrap(),
            }
        })
        .setup(move |app| {
            paths::ensure_elemento_dirs();

            let handle = app.handle().clone();
            let state = app_state.clone();
            launch_daemons(&handle, &state);

            // Hidden daemon log terminal (parity with electros-electron Terminal.js)
            let terminal_url = if cfg!(debug_assertions) {
                tauri::WebviewUrl::External(
                    "http://localhost:5173/terminal/terminal.html"
                        .parse()
                        .expect("terminal dev url"),
                )
            } else {
                tauri::WebviewUrl::App("terminal/terminal.html".into())
            };
            let _ = tauri::WebviewWindowBuilder::new(&handle, "daemon-terminal", terminal_url)
                .title("Electros Daemons")
                .inner_size(800.0, 600.0)
                .visible(false)
                .decorations(false)
                .zoom_hotkeys_enabled(true)
                .build();

            // System tray
            let _ = tauri::tray::TrayIconBuilder::new()
                .icon(handle.default_window_icon().unwrap().clone())
                .tooltip("Electros")
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(&handle);

            // Convert backgrounds on startup (async)
            let handle_bg = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = backgrounds::convert_existing_backgrounds().await;
                let _ = handle_bg;
            });

            // Deep link: initial + listen
            #[cfg(desktop)]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    for url in urls {
                        invoke::emit_deep_link(&app.handle(), url.to_string());
                    }
                }
                let app_handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        invoke::emit_deep_link(&app_handle, url.to_string());
                    }
                });
            }

            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // handled per-window in remotes
            }
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(move |_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                terminate_daemons(&app_state_on_exit);
            }
        });
}

#[tauri::command]
async fn electron_invoke(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    channel: String,
    args: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    invoke::dispatch(
        app,
        state.inner().clone(),
        InvokeRequest { channel, args },
    )
    .await
}

#[tauri::command]
async fn create_popup(
    app: tauri::AppHandle,
    options: serde_json::Value,
) -> Result<serde_json::Value, String> {
    commands::remotes::create_popup(app, options).await
}

#[tauri::command]
async fn electron_send(
    app: tauri::AppHandle,
    channel: String,
    args: Option<serde_json::Value>,
) -> Result<(), String> {
    invoke::electron_send(app, channel, args).await
}
