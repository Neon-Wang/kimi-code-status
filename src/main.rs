// Suppress dead_code warnings during initial development.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

mod config;
mod formatter;
mod harness;
mod keychain;
mod providers;
mod scheduler;
mod statusbar;
mod types;

use statusbar::{AppViewModel, StatusBarApp};

fn main() {
    env_logger::try_init().ok();

    let mtm = objc2_foundation::MainThreadMarker::new()
        .expect("must be started on the main thread");

    // Load configuration
    let cfg = config::load_config();
    let first_run = !cfg.first_run_completed;
    let selected_tools = if first_run {
        // Auto-select all installed tools on first run
        harness::scan_tools()
            .iter()
            .filter(|t| t.installed)
            .map(|t| t.id.clone())
            .collect()
    } else {
        cfg.selected_tools.clone()
    };

    // Scan for installed AI tools
    let tools = harness::scan_tools();
    log::info!(
        "Scanned {} tools, {} installed",
        tools.len(),
        tools.iter().filter(|t| t.installed).count()
    );

    // Create shared view model
    let vm = Arc::new(Mutex::new(AppViewModel {
        first_run,
        tools,
        selected_tools,
        ..Default::default()
    }));

    // Resolve icon path (try multiple locations)
    let icon_path = resolve_icon_path();

    // Build status bar (on main thread)
    let app = {
        let vm_guard = vm.lock().unwrap();
        StatusBarApp::new(mtm, &icon_path, &vm_guard)
    };
    let app = Arc::new(app);

    // If first run, mark it complete now
    if first_run {
        let mut new_cfg = cfg.clone();
        new_cfg.first_run_completed = true;
        new_cfg.selected_tools = {
            let vm_guard = vm.lock().unwrap();
            vm_guard.selected_tools.clone()
        };
        config::save_config(&new_cfg);
    }

    // Spawn background scheduler
    let app_clone = Arc::clone(&app);
    let vm_clone = Arc::clone(&vm);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            scheduler::run(app_clone, vm_clone).await;
        });
    });

    // Run the Cocoa event loop on the main thread.
    // The Arc is intentionally leaked — the app lives until Quit terminates the process.
    let _ = Arc::into_raw(app);

    let ns_app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    ns_app.run();
}

fn resolve_icon_path() -> String {
    // Try common locations for the icon
    let candidates = [
        // Running from project root (dev)
        "icons/statusbar_template.png".to_string(),
        // Running from .app bundle Resources/
        {
            if let Ok(exe) = std::env::current_exe() {
                if let Some(bundle) = exe
                    .parent() // MacOS/
                    .and_then(|p| p.parent()) // Contents/
                    .and_then(|p| p.parent()) // .app/
                {
                    format!(
                        "{}/Contents/Resources/statusbar_template.png",
                        bundle.display()
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        },
        // Running via cargo from cc-switch directory
        "../cc-switch/src-tauri/icons/tray/macos/statusbar_template_3x.png".to_string(),
    ];

    for path in &candidates {
        if !path.is_empty() && std::path::Path::new(path).exists() {
            log::info!("Using icon: {path}");
            return path.clone();
        }
    }

    // Fallback: return the first path anyway (will use text title)
    log::warn!("No icon found, using text fallback");
    candidates[0].clone()
}
