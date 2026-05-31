use crate::formatter;
use crate::types::{ServiceQuota, ToolInfo};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::rc::Retained;
use objc2::sel;
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem, NSStatusBar,
};

// objc2 0.6 traits for alloc
use objc2::AnyThread;
use objc2::MainThreadOnly;

// ── Shared state ──

#[derive(Default)]
pub struct AppViewModel {
    pub kimi_quota: Option<ServiceQuota>,
    pub codex_quota: Option<ServiceQuota>,
    pub tools: Vec<ToolInfo>,
    pub selected_tools: Vec<String>,
    pub first_run: bool,
}

impl AppViewModel {
    pub fn new() -> Self {
        Self::default()
    }
}

// ── StatusBarApp with raw pointers for cross-thread access ──

pub struct StatusBarApp {
    // Raw pointers to NSMenuItems we update dynamically.
    // Safe because all access is on the main thread via dispatch.
    kimi_item: *mut objc2::runtime::AnyObject,
    codex_item: *mut objc2::runtime::AnyObject,
    updated_item: *mut objc2::runtime::AnyObject,
    status_item: *mut objc2::runtime::AnyObject,
}

unsafe impl Send for StatusBarApp {}
unsafe impl Sync for StatusBarApp {}

impl StatusBarApp {
    /// Create the status bar and menu. Must be called on main thread.
    pub fn new(mtm: MainThreadMarker, icon_path: &str, vm: &AppViewModel) -> Self {
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let statusbar = NSStatusBar::systemStatusBar();
        let status_item =
            statusbar.statusItemWithLength(objc2_app_kit::NSVariableStatusItemLength);

        // Set icon on the status bar button
        unsafe {
            if let Some(button) = status_item.button(mtm) {
                let ns_path = NSString::from_str(icon_path);
                let image = NSImage::initWithContentsOfFile(NSImage::alloc(), &ns_path);
                if let Some(image) = image {
                    image.setTemplate(true);
                    // NSStatusBarButton inherits from NSButton → setImage:
                    let _: () = objc2::msg_send![
                        &*button,
                        setImage: &*image
                    ];
                } else {
                    let title = NSString::from_str("AI");
                    let _: () = objc2::msg_send![&*button, setTitle: &*title];
                }
            }
        }

        // Build initial menu
        let (kimi_item, codex_item, updated_item) = build_normal_menu(&status_item, mtm, vm);

        let status_raw: *mut objc2::runtime::AnyObject =
            &*status_item as *const objc2_app_kit::NSStatusItem as *mut _;

        Self {
            kimi_item,
            codex_item,
            updated_item,
            status_item: status_raw,
        }
    }

    /// Schedule a UI update from any thread.
    pub fn schedule_update(app: &Arc<Self>, vm: &Arc<Mutex<AppViewModel>>) {
        let app = Arc::clone(app);
        let vm = Arc::clone(vm);
        dispatch::Queue::main().exec_async(move || {
            let vm = vm.lock().unwrap();
            app.update_impl(&vm);
        });
    }

    /// Update labels on main thread. Do NOT call from background.
    fn update_impl(&self, vm: &AppViewModel) {
        unsafe {
            let kimi_label = Self::format_quota_line("Kimi Code", &vm.kimi_quota);
            let kimi_ns = NSString::from_str(&kimi_label);
            let _: () = objc2::msg_send![self.kimi_item, setTitle: &*kimi_ns];

            let codex_label = Self::format_quota_line("Codex", &vm.codex_quota);
            let codex_ns = NSString::from_str(&codex_label);
            let _: () = objc2::msg_send![self.codex_item, setTitle: &*codex_ns];

            let updated = Self::format_last_updated(vm);
            let updated_ns = NSString::from_str(&updated);
            let _: () = objc2::msg_send![self.updated_item, setTitle: &*updated_ns];

            // Update tooltip
            let worst = Self::worst_status(vm);
            let worst_ns = NSString::from_str(&worst);
            // Get button via the NSStatusItem reference we stored as a raw pointer.
            // Reconstruct a reference (valid because NSStatusItem is retained by NSStatusBar).
            let status_item: &objc2_app_kit::NSStatusItem =
                &*(self.status_item as *const objc2_app_kit::NSStatusItem);
            let mtm = MainThreadMarker::new().expect("update must be on main thread");
            if let Some(button) = status_item.button(mtm) {
                let _: () = objc2::msg_send![&*button, setToolTip: &*worst_ns];
            }
        }
    }

    // ── Formatting helpers ──

    fn format_quota_line(name: &str, quota: &Option<ServiceQuota>) -> String {
        match quota {
            Some(q) if q.success => {
                let summary = formatter::format_summary(&q.tiers)
                    .unwrap_or_else(|| "No data".into());
                format!("  {name}  {summary}")
            }
            Some(q) if !q.credential_valid => {
                let err = q.error.as_deref().unwrap_or("Not configured");
                format!("  {name}  \u{26A0} {err}")
            }
            Some(q) => {
                let err = q.error.as_deref().unwrap_or("Query failed");
                format!("  {name}  \u{26A0} {err}")
            }
            None => format!("  {name}  \u{26AA} Loading..."),
        }
    }

    fn format_last_updated(vm: &AppViewModel) -> String {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let latest = [&vm.kimi_quota, &vm.codex_quota]
            .iter()
            .filter_map(|q| q.as_ref())
            .filter_map(|q| q.queried_at)
            .max()
            .unwrap_or(0);

        if latest == 0 {
            "  Last updated: never".into()
        } else {
            let secs_ago = ((now_ms - latest) as f64 / 1000.0) as u64;
            let time_str = if secs_ago < 60 {
                format!("{secs_ago}s ago")
            } else if secs_ago < 3600 {
                format!("{}m ago", secs_ago / 60)
            } else {
                format!("{}h ago", secs_ago / 3600)
            };
            format!("  Last updated: {time_str}")
        }
    }

    fn worst_status(vm: &AppViewModel) -> String {
        let mut worst_pct: Option<f64> = None;
        let mut has_data = false;

        for q in [&vm.kimi_quota, &vm.codex_quota].iter().filter_map(|q| q.as_ref()) {
            if q.success {
                for t in &q.tiers {
                    has_data = true;
                    let prev = worst_pct.unwrap_or(0.0);
                    if t.utilization > prev {
                        worst_pct = Some(t.utilization);
                    }
                }
            }
        }

        if !has_data {
            "AI Coding Dashboard".into()
        } else {
            let pct = worst_pct.unwrap_or(0.0);
            let emoji = formatter::emoji_for_utilization(pct);
            format!("{emoji} {:.0}% used", pct)
        }
    }
}

// ── Menu builder ──

fn build_normal_menu(
    status_item: &objc2_app_kit::NSStatusItem,
    mtm: MainThreadMarker,
    vm: &AppViewModel,
) -> (
    *mut objc2::runtime::AnyObject,
    *mut objc2::runtime::AnyObject,
    *mut objc2::runtime::AnyObject,
) {
    unsafe {
        let menu = build_menu(mtm, vm);

        // Items at known indices:
        // 0: "Services" header, 1: Kimi, 2: Codex, 3: Updated, 4: sep,
        // 5: "Harness Tools" header, ...tool items..., then separator, actions, quit.
        let kimi: *mut objc2::runtime::AnyObject =
            objc2::msg_send![&*menu, itemAtIndex: 1_isize];
        let codex: *mut objc2::runtime::AnyObject =
            objc2::msg_send![&*menu, itemAtIndex: 2_isize];
        let updated: *mut objc2::runtime::AnyObject =
            objc2::msg_send![&*menu, itemAtIndex: 3_isize];

        // itemAtIndex returns an autoreleased object; we need to retain it
        // so it stays valid after the menu reference is gone.
        if !kimi.is_null() {
            let _: () = objc2::msg_send![kimi, retain];
        }
        if !codex.is_null() {
            let _: () = objc2::msg_send![codex, retain];
        }
        if !updated.is_null() {
            let _: () = objc2::msg_send![updated, retain];
        }

        let _: () = objc2::msg_send![
            status_item as *const objc2_app_kit::NSStatusItem as *mut objc2::runtime::AnyObject,
            setMenu: &*menu
        ];

        (kimi, codex, updated)
    }
}

unsafe fn build_menu(mtm: MainThreadMarker, vm: &AppViewModel) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);

    // ── Services section ──
    add_disabled_item(&menu, mtm, "Services");

    let kimi_label = StatusBarApp::format_quota_line("Kimi Code", &vm.kimi_quota);
    add_disabled_item(&menu, mtm, &kimi_label);

    let codex_label = StatusBarApp::format_quota_line("Codex", &vm.codex_quota);
    add_disabled_item(&menu, mtm, &codex_label);

    let updated = StatusBarApp::format_last_updated(vm);
    add_disabled_item(&menu, mtm, &updated);

    add_separator(&menu, mtm);

    // ── Harness Tools section ──
    add_disabled_item(&menu, mtm, "Harness Tools");

    for tool in &vm.tools {
        let prefix = if tool.installed { "  \u{2713}" } else { "  \u{2717}" };
        let label = format!("{prefix} {}", tool.name);
        add_disabled_item(&menu, mtm, &label);
    }

    add_separator(&menu, mtm);

    // ── Actions ──
    add_action_item(&menu, mtm, "Refresh All", sel!(refreshAll:));
    add_action_item(&menu, mtm, "Manage Services...", sel!(manageServices:));

    add_separator(&menu, mtm);

    add_action_item(&menu, mtm, "Quit", sel!(quit:));

    menu
}

unsafe fn add_disabled_item(menu: &NSMenu, mtm: MainThreadMarker, title: &str) {
    let ns_title = NSString::from_str(title);
    let item = NSMenuItem::initWithTitle_action_keyEquivalent(
        NSMenuItem::alloc(mtm),
        &ns_title,
        None,
        &NSString::from_str(""),
    );
    item.setEnabled(false);
    menu.addItem(&item);
}

unsafe fn add_action_item(menu: &NSMenu, mtm: MainThreadMarker, title: &str, action: objc2::runtime::Sel) {
    let ns_title = NSString::from_str(title);
    let item = NSMenuItem::initWithTitle_action_keyEquivalent(
        NSMenuItem::alloc(mtm),
        &ns_title,
        Some(action),
        &NSString::from_str(""),
    );
    item.setEnabled(true);
    menu.addItem(&item);
}

unsafe fn add_separator(menu: &NSMenu, mtm: MainThreadMarker) {
    let sep = NSMenuItem::separatorItem(mtm);
    menu.addItem(&sep);
}
