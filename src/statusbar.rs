use crate::formatter;
use crate::types::{ServiceQuota, ToolInfo};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::rc::Retained;
use objc2::sel;
use objc2::{MainThreadOnly};
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSMenu, NSMenuItem, NSStatusBar,
};

// ── Shared state ──

#[derive(Default)]
pub struct AppViewModel {
    pub kimi_quota: Option<ServiceQuota>,
    pub codex_quota: Option<ServiceQuota>,
    pub tools: Vec<ToolInfo>,
    pub selected_tools: Vec<String>,
    pub first_run: bool,
}

pub struct StatusBarApp {
    status_item: Retained<objc2_app_kit::NSStatusItem>,
}

unsafe impl Send for StatusBarApp {}
unsafe impl Sync for StatusBarApp {}

impl StatusBarApp {
    pub fn new(mtm: MainThreadMarker, vm: &AppViewModel) -> Self {
        NSApplication::sharedApplication(mtm)
            .setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let statusbar = NSStatusBar::systemStatusBar();
        let status_item =
            statusbar.statusItemWithLength(objc2_app_kit::NSVariableStatusItemLength);

        // Show usage text directly in the menu bar (no icon)
        if let Some(button) = status_item.button(mtm) {
            unsafe {
                let text = Self::menu_bar_text(vm);
                let _: () = objc2::msg_send![&*button, setTitle: &*NSString::from_str(&text)];
                let _: () = objc2::msg_send![&*button, setToolTip: &*NSString::from_str("AI Coding Dashboard — click for details")];
            }
        }

        // Build menu
        let menu = Self::build_menu(mtm, vm);
        status_item.setMenu(Some(&menu));

        Self { status_item }
    }

    pub fn schedule_update(app: &Arc<Self>, vm: &Arc<Mutex<AppViewModel>>) {
        let app = Arc::clone(app);
        let vm = Arc::clone(vm);
        dispatch::Queue::main().exec_async(move || {
            if let Ok(vm) = vm.lock() {
                app.update(&vm);
            }
        });
    }

    fn update(&self, vm: &AppViewModel) {
        let mtm = MainThreadMarker::new().expect("main thread");
        if let Some(button) = self.status_item.button(mtm) {
            unsafe {
                let text = Self::menu_bar_text(vm);
                let _: () = objc2::msg_send![&*button, setTitle: &*NSString::from_str(&text)];
            }
        }
        let menu = Self::build_menu(mtm, vm);
        self.status_item.setMenu(Some(&menu));
    }

    fn menu_bar_text(vm: &AppViewModel) -> String {
        let mut parts = Vec::new();
        for q in [&vm.kimi_quota, &vm.codex_quota] {
            match q {
                Some(q) if q.success => {
                    let emoji = formatter::status_emoji(&q.tiers);
                    let max_pct = q.tiers.iter().map(|t| t.utilization).fold(0.0f64, f64::max);
                    parts.push(format!("{emoji}{:.0}%", max_pct));
                }
                Some(q) if !q.credential_valid && q.service == "kimi" => {
                    parts.push("\u{26A0} Set Key".into());
                }
                _ => {}
            }
        }
        if parts.is_empty() { "\u{26AA} KCode".into() } else { parts.join(" ") }
    }

    fn build_menu(mtm: MainThreadMarker, vm: &AppViewModel) -> Retained<NSMenu> {
        let menu = NSMenu::new(mtm);

        // ── Usage ──
        Self::dlbl(&menu, mtm, "Usage");
        Self::dlbl(&menu, mtm, &format!("  Kimi Code  {}", Self::qdetail(vm.kimi_quota.as_ref())));
        Self::dlbl(&menu, mtm, &format!("  Codex       {}", Self::qdetail(vm.codex_quota.as_ref())));
        Self::dlbl(&menu, mtm, &Self::fmt_updated(vm));
        Self::sep(&menu, mtm);

        // ── API Keys ──
        Self::dlbl(&menu, mtm, "API Keys");
        let has_kimi = vm.kimi_quota.as_ref().is_some_and(|q| q.credential_valid);
        Self::dlbl(&menu, mtm, &format!("  Kimi Code  {}", if has_kimi { "\u{2713} configured" } else { "\u{26A0} Set via Keychain" }));
        let has_codex = vm.codex_quota.as_ref().is_some_and(|q| q.credential_valid);
        Self::dlbl(&menu, mtm, &format!("  Codex       {}", if has_codex { "\u{2713} auto-detected" } else { "\u{26A0} auto-detect: none" }));
        Self::dlbl(&menu, mtm, "       Run: security add-generic-password -s \"Kimi Code Status\" -a \"kimi-api-key\" -w \"sk-...\"");
        Self::sep(&menu, mtm);

        // ── Harness Tools ──
        Self::dlbl(&menu, mtm, "Harness Tools");
        for tool in &vm.tools {
            let icon = if tool.installed { "  \u{2713}" } else { "  \u{2717}" };
            Self::dlbl(&menu, mtm, &format!("{icon} {}", tool.name));
        }
        Self::sep(&menu, mtm);

        // ── Actions ──
        Self::act(&menu, mtm, "Refresh Now", None);
        Self::sep(&menu, mtm);
        Self::act(&menu, mtm, "Quit", Some(sel!(terminate:)));

        menu
    }

    fn qdetail(q: Option<&ServiceQuota>) -> String {
        match q {
            Some(q) if q.success => formatter::format_summary(&q.tiers).unwrap_or("No data".into()),
            Some(q) if !q.credential_valid => format!("\u{26A0} {}", q.error.as_deref().unwrap_or("Not configured")),
            Some(q) => format!("\u{26A0} {}", q.error.as_deref().unwrap_or("Query failed")),
            None => "\u{26AA} Loading...".into(),
        }
    }

    fn fmt_updated(vm: &AppViewModel) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64;
        let latest = [&vm.kimi_quota, &vm.codex_quota].iter()
            .filter_map(|q| q.as_ref()).filter_map(|q| q.queried_at).max().unwrap_or(0);
        if latest == 0 { "  Last updated: never".into() } else {
            let s = ((now - latest) as f64 / 1000.0) as u64;
            let t = if s < 60 { format!("{s}s ago") } else if s < 3600 { format!("{}m ago", s/60) } else { format!("{}h ago", s/3600) };
            format!("  Last updated: {t}")
        }
    }

    // ── Menu helpers ──

    fn dlbl(menu: &NSMenu, mtm: MainThreadMarker, title: &str) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                None,
                &NSString::from_str(""),
            )
        };
        item.setEnabled(false);
        menu.addItem(&item);
    }

    fn act(menu: &NSMenu, mtm: MainThreadMarker, title: &str, action: Option<objc2::runtime::Sel>) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                action,
                &NSString::from_str(""),
            )
        };
        item.setEnabled(true);
        menu.addItem(&item);
    }

    fn sep(menu: &NSMenu, mtm: MainThreadMarker) {
        menu.addItem(&NSMenuItem::separatorItem(mtm));
    }
}
