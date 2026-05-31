use crate::config;
use crate::providers::{codex::CodexProvider, kimi::KimiProvider, UsageProvider};
use crate::statusbar::{AppViewModel, StatusBarApp};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const REFRESH_INTERVAL_SECS: u64 = 300; // 5 minutes

/// Run the scheduler loop on a background tokio runtime.
/// Sends updates to the main thread via dispatch.
pub async fn run(app: Arc<StatusBarApp>, vm: Arc<Mutex<AppViewModel>>) {
    let config = config::load_config();
    let kimi_enabled = config.selected_services.contains(&"kimi".to_string());
    let codex_enabled = config.selected_services.contains(&"codex".to_string());

    // Initial query
    refresh_all(kimi_enabled, codex_enabled, &app, &vm).await;

    // Periodic refresh
    let mut interval = tokio::time::interval(Duration::from_secs(REFRESH_INTERVAL_SECS));
    loop {
        interval.tick().await;
        log::info!("Scheduler tick — refreshing usage data");
        let config = config::load_config();
        let kimi_enabled = config.selected_services.contains(&"kimi".to_string());
        let codex_enabled = config.selected_services.contains(&"codex".to_string());
        refresh_all(kimi_enabled, codex_enabled, &app, &vm).await;
    }
}

async fn refresh_all(
    kimi_enabled: bool,
    codex_enabled: bool,
    app: &Arc<StatusBarApp>,
    vm: &Arc<Mutex<AppViewModel>>,
) {
    // Query Kimi
    if kimi_enabled {
        let kimi = KimiProvider::new();
        let quota = kimi.query().await;

        {
            let mut vm = vm.lock().unwrap();
            vm.kimi_quota = Some(quota);
        }
    }

    // Query Codex
    if codex_enabled {
        let codex = CodexProvider::new();
        let quota = codex.query().await;

        {
            let mut vm = vm.lock().unwrap();
            vm.codex_quota = Some(quota);
        }
    }

    // Dispatch UI update to main thread
    StatusBarApp::schedule_update(app, vm);
}
