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

use objc2_foundation::MainThreadMarker;
use objc2_app_kit::NSApplication;

use statusbar::{AppViewModel, StatusBarApp};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("AI Coding Dashboard starting...");

    let mtm = MainThreadMarker::new().expect("must run on main thread");

    let cfg = config::load_config();
    let tools = harness::scan_tools();
    let selected_tools = cfg.selected_tools.clone();

    let vm = Arc::new(Mutex::new(AppViewModel {
        first_run: !cfg.first_run_completed,
        tools,
        selected_tools,
        ..Default::default()
    }));

    let app = {
        let vm_guard = vm.lock().unwrap();
        StatusBarApp::new(mtm, &vm_guard)
    };

    let app = Arc::new(app);

    // Background scheduler
    let app_clone = Arc::clone(&app);
    let vm_clone = Arc::clone(&vm);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(scheduler::run(app_clone, vm_clone));
    });

    // Event loop
    let ns_app = NSApplication::sharedApplication(mtm);
    Box::leak(Box::new(app));
    drop(vm);
    ns_app.run();
}
