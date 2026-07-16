use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use culinograph_service::{AccessPolicy, ServiceConfig, ServiceState, bind};
use serde::Serialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tauri::Manager;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Write a base64-encoded export bundle to a path the user chose via the native
/// save dialog. Writing on the Rust side keeps us clear of filesystem-scope
/// permissions while still honoring an arbitrary destination.
#[tauri::command]
fn save_export(path: String, contents_base64: String) -> Result<(), String> {
    let bytes = BASE64
        .decode(contents_base64.as_bytes())
        .map_err(|error| format!("Could not decode export bundle: {error}"))?;
    std::fs::write(PathBuf::from(path), bytes)
        .map_err(|error| format!("Could not write export file: {error}"))
}

/// Read a recipe DSL source file the user chose via the native open dialog.
/// Reading on the Rust side keeps us clear of filesystem-scope permissions.
#[tauri::command]
fn read_recipe_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(PathBuf::from(path))
        .map_err(|error| format!("Could not read recipe file: {error}"))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceBootstrap {
    endpoint: String,
    websocket_url: String,
    token: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shutdown = CancellationToken::new();
    let shutdown_for_setup = shutdown.clone();
    let bootstrap = Arc::new(RwLock::new(None::<ServiceBootstrap>));
    let bootstrap_for_setup = Arc::clone(&bootstrap);
    let bootstrap_for_page_load = Arc::clone(&bootstrap);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![save_export, read_recipe_file])
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database_path = data_dir.join("culinograph.sqlite3");
            let settings_path = data_dir.join("settings.json");

            let token = Uuid::new_v4().simple().to_string();
            let allowed_origins = vec![
                "tauri://localhost".to_owned(),
                "http://tauri.localhost".to_owned(),
                "https://tauri.localhost".to_owned(),
                "http://localhost:1420".to_owned(),
            ];
            let state = ServiceState::sqlite(database_path, settings_path)?;
            if let Err(error) = state.seed_if_empty() {
                eprintln!("Culinograph sample recipes were not seeded: {error}");
            }
            let service = tauri::async_runtime::block_on(bind(
                ServiceConfig {
                    state,
                    access: AccessPolicy::new(token.clone(), allowed_origins.clone()),
                    allowed_origins,
                },
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            ))?;
            let address = service.local_addr()?;
            let service_bootstrap = ServiceBootstrap {
                endpoint: format!("http://{address}"),
                websocket_url: format!("ws://{address}/ws"),
                token,
            };
            *bootstrap_for_setup
                .write()
                .map_err(|_| std::io::Error::other("service bootstrap lock poisoned"))? = Some(service_bootstrap);

            let shutdown = shutdown_for_setup.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = service.serve(shutdown).await {
                    eprintln!("Culinograph local service failed: {error}");
                }
            });
            Ok(())
        })
        .on_page_load(move |webview, _payload| {
            let Ok(guard) = bootstrap_for_page_load.read() else {
                return;
            };
            let Some(value) = guard.as_ref() else {
                return;
            };
            let Ok(json) = serde_json::to_string(value) else {
                return;
            };
            let script = format!(
                "window.__CULINOGRAPH_SERVICE__ = {json}; window.dispatchEvent(new CustomEvent('culinograph:service-ready', {{ detail: {json} }}));"
            );
            if let Err(error) = webview.eval(&script) {
                eprintln!("Could not inject Culinograph service bootstrap: {error}");
            }
        })
        .on_window_event(move |_window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                shutdown.cancel();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Culinograph");
}
#[cfg(test)]
mod test;
