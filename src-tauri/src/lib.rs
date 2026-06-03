#![allow(dead_code)]

mod commands;
mod config;
mod credentials;
mod estimator;
mod harness;
mod keychain;
mod launcher;
mod providers;
mod proxy;
mod scheduler;
mod tray;
mod types;
mod vault;

use commands::{AppRuntimeState, SharedRuntimeState};
use std::sync::{Arc, Mutex};
use tauri::{ActivationPolicy, Manager};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(Arc::new(Mutex::new(AppRuntimeState::default())) as SharedRuntimeState)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                let app = window.app_handle();
                let _ = app.set_dock_visibility(false);
                let _ = app.set_activation_policy(ActivationPolicy::Accessory);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard_state,
            commands::refresh_usage,
            commands::set_selected_tools,
            commands::set_selected_services,
            commands::save_proxy_settings,
            commands::test_proxy,
            commands::save_kimi_api_key,
            commands::clear_kimi_api_key,
            commands::launch_tool,
            commands::reveal_config_dir
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<SharedRuntimeState>().inner().clone();
            let dashboard = commands::dashboard_state(&state)?;
            tray::create_tray(&handle, &dashboard)?;
            tauri::async_runtime::spawn(async move {
                if let Err(error) = commands::refresh_usage_inner(&state).await {
                    log::warn!("Initial usage refresh failed: {error}");
                }
                if let Ok(dashboard) = commands::dashboard_state(&state) {
                    let _ = tray::update_tray(&handle, &dashboard);
                    commands::emit_dashboard_update(&handle, &dashboard);
                }
            });
            let handle = app.handle().clone();
            let state = app.state::<SharedRuntimeState>().inner().clone();
            tauri::async_runtime::spawn(scheduler::run(handle, state));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
