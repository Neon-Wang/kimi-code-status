use std::time::Duration;
use tauri::AppHandle;

const REFRESH_INTERVAL_SECS: u64 = 300; // 5 minutes

pub async fn run(app: AppHandle, state: crate::commands::SharedRuntimeState) {
    loop {
        tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
        if let Err(error) = refresh_and_publish(&app, &state).await {
            log::warn!("Scheduled usage refresh failed: {error}");
        }
    }
}

async fn refresh_and_publish(
    app: &AppHandle,
    state: &crate::commands::SharedRuntimeState,
) -> Result<(), String> {
    log::info!("Scheduled usage refresh started");
    crate::commands::refresh_usage_inner(state).await?;
    let dashboard = crate::commands::dashboard_state(state)?;
    crate::tray::update_tray(app, &dashboard)?;
    crate::commands::emit_dashboard_update(app, &dashboard);
    log::info!("Scheduled usage refresh completed");
    Ok(())
}
