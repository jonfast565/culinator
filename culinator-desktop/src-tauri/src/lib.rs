use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use culinator_service::{AccessPolicy, ServiceConfig, ServiceState, bind};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tauri::{
    AppHandle, Emitter, Manager, Wry,
    menu::{Menu, MenuItemBuilder, Submenu, SubmenuBuilder},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Event carrying the id of a chosen native menu item to the webview. The id is
/// the frontend's own `AppMenuAction`, so the shell never interprets commands.
const MENU_ACTION_EVENT: &str = "culinator://menu-action";

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

/// One top-level menu, as described by the frontend's menu model.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MenuSectionSpec {
    label: String,
    enabled: bool,
    items: Vec<MenuItemSpec>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MenuItemSpec {
    /// The frontend `AppMenuAction`, used verbatim as the menu item id.
    id: String,
    label: String,
    enabled: bool,
    /// Draw a separator above this item.
    separator_before: bool,
    accelerator: Option<String>,
}

/// Replace the native menu with the frontend's current one.
///
/// Menu labels track app state ("Use US units" / "Use metric units"), so the
/// menu is rebuilt on every change rather than declared once here. Building it
/// is a main-thread operation on macOS, hence the hop.
#[tauri::command]
fn set_app_menu(app: AppHandle, menu: Vec<MenuSectionSpec>) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if let Err(error) = install_app_menu(&handle, &menu) {
            eprintln!("Could not install the native menu: {error}");
        }
    })
    .map_err(|error| format!("Could not reach the UI thread: {error}"))
}

fn install_app_menu(app: &AppHandle, sections: &[MenuSectionSpec]) -> tauri::Result<()> {
    app.set_menu(build_app_menu(app, sections)?)?;
    Ok(())
}

fn build_app_menu(app: &AppHandle, sections: &[MenuSectionSpec]) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;

    #[cfg(target_os = "macos")]
    menu.append(
        &SubmenuBuilder::new(app, "Culinator")
            .about(None)
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?,
    )?;

    for (index, section) in sections.iter().enumerate() {
        menu.append(&build_section(app, section)?)?;
        // Replacing the default menu takes the webview's clipboard and undo
        // shortcuts with it, so the standard Edit menu goes back in — in its
        // conventional place, right after File.
        if index == 0 {
            menu.append(&edit_submenu(app)?)?;
        }
    }
    if sections.is_empty() {
        menu.append(&edit_submenu(app)?)?;
    }

    #[cfg(target_os = "macos")]
    menu.append(
        &SubmenuBuilder::new(app, "Window")
            .minimize()
            .maximize()
            .fullscreen()
            .separator()
            .close_window()
            .build()?,
    )?;

    Ok(menu)
}

fn build_section(app: &AppHandle, section: &MenuSectionSpec) -> tauri::Result<Submenu<Wry>> {
    let mut builder = SubmenuBuilder::new(app, &section.label).enabled(section.enabled);
    for item in &section.items {
        if item.separator_before {
            builder = builder.separator();
        }
        let mut entry = MenuItemBuilder::with_id(item.id.clone(), &item.label)
            .enabled(section.enabled && item.enabled);
        if let Some(accelerator) = &item.accelerator {
            entry = entry.accelerator(accelerator);
        }
        builder = builder.item(&entry.build(app)?);
    }
    builder.build()
}

fn edit_submenu(app: &AppHandle) -> tauri::Result<Submenu<Wry>> {
    SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()
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
        .invoke_handler(tauri::generate_handler![
            save_export,
            read_recipe_file,
            set_app_menu
        ])
        .on_menu_event(|app, event| {
            // Menu ids are frontend actions; the shell just forwards them.
            if let Err(error) = app.emit(MENU_ACTION_EVENT, event.id().as_ref()) {
                eprintln!("Could not forward menu action: {error}");
            }
        })
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database_path = data_dir.join("culinator.sqlite3");
            let settings_path = data_dir.join("settings.json");

            let token = Uuid::new_v4().simple().to_string();
            let allowed_origins = vec![
                "tauri://localhost".to_owned(),
                "http://tauri.localhost".to_owned(),
                "https://tauri.localhost".to_owned(),
                "http://localhost:1420".to_owned(),
            ];
            let state = ServiceState::sqlite(database_path, settings_path)?;
            // Sample seeding runs via service.initialize once the UI connects.
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
                    eprintln!("Culinator local service failed: {error}");
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
                "window.__CULINATOR_SERVICE__ = {json}; window.dispatchEvent(new CustomEvent('culinator:service-ready', {{ detail: {json} }}));"
            );
            if let Err(error) = webview.eval(&script) {
                eprintln!("Could not inject Culinator service bootstrap: {error}");
            }
        })
        .on_window_event(move |_window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                shutdown.cancel();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Culinator");
}
#[cfg(test)]
mod test;
