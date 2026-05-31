use crate::formatter;
use crate::types::{QuotaTier, ServiceQuota, ToolInfo};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::sel;
use objc2::MainThreadOnly;
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
    _delegate: Retained<AnyObject>,
}

unsafe impl Send for StatusBarApp {}
unsafe impl Sync for StatusBarApp {}

// ── Raw ObjC delegate class (no objc2 ClassBuilder) ──

fn make_delegate() -> Retained<AnyObject> {
    // Lazy-init the class on first call
    static CLASS: std::sync::OnceLock<SendClass> = std::sync::OnceLock::new();
    let cls_ptr = CLASS.get_or_init(|| {
        let name = std::ffi::CString::new(format!("KCSDel{}", std::process::id())).unwrap();
        unsafe {
            let superclass: *mut AnyObject = AnyClass::get(c"NSObject").unwrap() as *const _ as *mut _;
            let cls = objc_allocateClassPair(superclass, name.as_ptr(), 0);
            if cls.is_null() { panic!("objc_allocateClassPair failed"); }

            let s = sel!(openTool:);
            class_addMethod(cls, s, open_tool_impl as *mut c_void, c"v@:@".as_ptr());
            let s = sel!(refreshNow:);
            class_addMethod(cls, s, refresh_now_impl as *mut c_void, c"v@:@".as_ptr());
            let s = sel!(setKimiKey:);
            class_addMethod(cls, s, set_key_impl as *mut c_void, c"v@:@".as_ptr());

            objc_registerClassPair(cls);
            SendClass(cls)
        }
    }).0;

    unsafe {
        let obj: *mut AnyObject = objc2::msg_send![cls_ptr, alloc];
        let obj: *mut AnyObject = objc2::msg_send![obj, init];
        Retained::from_raw(obj).expect("delegate alloc+init")
    }
}

extern "C" {
    fn objc_allocateClassPair(
        superclass: *mut AnyObject,
        name: *const std::os::raw::c_char,
        extra_bytes: usize,
    ) -> *mut AnyObject;
    fn objc_registerClassPair(cls: *mut AnyObject);
    fn class_addMethod(
        cls: *mut AnyObject,
        name: Sel,
        imp: *mut c_void,
        types: *const std::os::raw::c_char,
    ) -> bool;
}

// Wrapper to make raw class pointer Send+Sync (safe: ObjC classes are global)
struct SendClass(*mut AnyObject);
unsafe impl Send for SendClass {}
unsafe impl Sync for SendClass {}

// ── Delegate method implementations ──

extern "C" fn open_tool_impl(_this: &AnyObject, _cmd: Sel, sender: *mut AnyObject) {
    if sender.is_null() { return; }
    // Read representedObject (NSString) from the menu item
    let obj: *mut AnyObject = unsafe { objc2::msg_send![sender, representedObject] };
    if obj.is_null() { return; }
    let name = unsafe { (*(obj as *const NSString)).to_string() };
    // Launch via open -a (fire-and-forget)
    std::thread::spawn(move || {
        let _ = std::process::Command::new("open")
            .arg("-a").arg(&name)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    });
}

extern "C" fn refresh_now_impl(_this: &AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
    // Just show a note — auto-refresh handles it
    show_alert_str("Refresh", "Auto-refresh runs every 5 minutes.\nQueries are in progress now.");
}

extern "C" fn set_key_impl(_this: &AnyObject, _cmd: Sel, _sender: *mut AnyObject) {
    show_alert_str(
        "Set Kimi API Key",
        "Run this command in Terminal:\n\nsecurity add-generic-password -s \"Kimi Code Status\" -a \"kimi-api-key\" -w \"sk-your-key\"\n\nThen restart the app.",
    );
}

fn show_alert_str(title: &str, msg: &str) {
    // Use NSAlert on main thread. We're called from a menu action which is already on main thread.
    unsafe {
        let cls: *mut AnyObject = AnyClass::get(c"NSAlert").unwrap() as *const _ as *mut _;
        let a: *mut AnyObject = objc2::msg_send![cls, new];
        let _: () = objc2::msg_send![a, setMessageText: &*NSString::from_str(title)];
        let _: () = objc2::msg_send![a, setInformativeText: &*NSString::from_str(msg)];
        let _: () = objc2::msg_send![a, addButtonWithTitle: &*NSString::from_str("OK")];
        let _: () = objc2::msg_send![a, runModal];
    }
}

// ── StatusBarApp impl ──

impl StatusBarApp {
    pub fn new(mtm: MainThreadMarker, vm: &AppViewModel) -> Self {
        NSApplication::sharedApplication(mtm)
            .setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let delegate = make_delegate();
        let del_ptr: *mut AnyObject = &*delegate as *const _ as *mut _;

        let statusbar = NSStatusBar::systemStatusBar();
        let status_item =
            statusbar.statusItemWithLength(objc2_app_kit::NSVariableStatusItemLength);

        // Show usage text in menu bar
        if let Some(button) = status_item.button(mtm) {
            unsafe {
                let text = Self::menu_bar_text(vm);
                let _: () = objc2::msg_send![&*button, setTitle: &*NSString::from_str(&text)];
                let _: () = objc2::msg_send![&*button, setToolTip: &*NSString::from_str("AI Coding Dashboard — click for details")];
            }
        }

        // Build initial menu
        let menu = Self::build_menu(mtm, vm, del_ptr);
        status_item.setMenu(Some(&menu));

        Self { status_item, _delegate: delegate }
    }

    pub fn schedule_update(app: &Arc<Self>, vm: &Arc<Mutex<AppViewModel>>) {
        let app = Arc::clone(app);
        let vm = Arc::clone(vm);
        dispatch::Queue::main().exec_async(move || {
            if let Ok(guard) = vm.lock() {
                app.update(&guard);
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
        // Rebuild menu with data
        let del_ptr: *mut AnyObject = &*self._delegate as *const _ as *mut _;
        let menu = Self::build_menu(mtm, vm, del_ptr);
        self.status_item.setMenu(Some(&menu));
    }

    fn menu_bar_text(vm: &AppViewModel) -> String {
        let mut parts = Vec::new();
        for q in [&vm.kimi_quota, &vm.codex_quota] {
            match q {
                Some(q) if q.success => {
                    if let Some(s) = formatter::format_summary(&q.tiers) { parts.push(s); }
                }
                Some(q) if !q.credential_valid && q.service == "kimi" => {
                    parts.push("\u{26A0} Set Key".into());
                }
                _ => {}
            }
        }
        if parts.is_empty() { "\u{26AA} KCode".into() } else { parts.join("  ") }
    }

    fn build_menu(mtm: MainThreadMarker, vm: &AppViewModel, del_ptr: *mut AnyObject) -> Retained<NSMenu> {
        let menu = NSMenu::new(mtm);

        // ── Usage ──
        Self::dlbl(&menu, mtm, "Usage");
        Self::service_lines(&menu, mtm, "Kimi Code", &vm.kimi_quota);
        Self::service_lines(&menu, mtm, "Codex", &vm.codex_quota);
        Self::dlbl(&menu, mtm, &Self::fmt_updated(vm));
        Self::sep(&menu, mtm);

        // ── API Keys ──
        Self::dlbl(&menu, mtm, "API Keys");
        let has_kimi = vm.kimi_quota.as_ref().is_some_and(|q| q.credential_valid);
        if has_kimi {
            Self::dlbl(&menu, mtm, "  Kimi Code  \u{2713} configured");
        } else {
            Self::act_with_target(&menu, mtm, "  Kimi Code  \u{26A0} Set API Key...", sel!(setKimiKey:), del_ptr);
        }
        let has_codex = vm.codex_quota.as_ref().is_some_and(|q| q.credential_valid);
        Self::dlbl(&menu, mtm, &format!("  Codex       {}", if has_codex { "\u{2713} auto-detected" } else { "\u{26A0} auto-detect: none" }));
        Self::sep(&menu, mtm);

        // ── Harness Tools (clickable to launch!) ──
        Self::dlbl(&menu, mtm, "Harness Tools");
        for tool in &vm.tools {
            let icon = if tool.installed { "  \u{2713}" } else { "  \u{2717}" };
            let label = format!("{icon} {}", tool.name);
            if tool.installed {
                let launch_name = tool.launch_as.as_deref().unwrap_or(&tool.name);
                Self::act_with_target_and_tag(&menu, mtm, &label, sel!(openTool:), del_ptr, launch_name);
            } else {
                Self::dlbl(&menu, mtm, &label);
            }
        }
        Self::sep(&menu, mtm);

        // ── Actions ──
        Self::act_with_target(&menu, mtm, "Refresh Now", sel!(refreshNow:), del_ptr);
        Self::sep(&menu, mtm);
        Self::act(&menu, mtm, "Quit", Some(sel!(terminate:)));

        menu
    }

    fn service_lines(menu: &NSMenu, mtm: MainThreadMarker, name: &str, q: &Option<ServiceQuota>) {
        match q {
            Some(q) if q.success && !q.tiers.is_empty() => {
                for t in &q.tiers {
                    let label = match t.name.as_str() {
                        "five_hour" => "5-Hour",
                        "weekly_limit" | "seven_day" => "7-Day",
                        _ => &t.name,
                    };
                    let pct = format!("{:.0}%", t.utilization);
                    let cd = Self::tier_countdown(t);
                    let line = if cd.is_empty() {
                        format!("    {label}: {pct}")
                    } else {
                        format!("    {label}: {pct}  (resets {cd})")
                    };
                    Self::dlbl(menu, mtm, &line);
                }
            }
            Some(_) => { Self::dlbl(menu, mtm, &format!("  {}  No data", name)); }
            None => { Self::dlbl(menu, mtm, &format!("  {}  \u{26AA} Loading...", name)); }
        }
    }

    fn tier_countdown(t: &QuotaTier) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        if let Some(ref r) = t.resets_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(r) {
                let rem = dt.timestamp() - now;
                if rem <= 0 { return String::new(); }
                if rem < 3600 { format!("{}m", rem/60) }
                else if rem < 86400 { format!("{}h{}m", rem/3600, (rem%3600)/60) }
                else { format!("{}d{}h", rem/86400, (rem%86400)/3600) }
            } else { String::new() }
        } else { String::new() }
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
            NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(title), None, &NSString::from_str(""))
        };
        item.setEnabled(false);
        menu.addItem(&item);
    }

    fn act(menu: &NSMenu, mtm: MainThreadMarker, title: &str, action: Option<Sel>) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(title), action, &NSString::from_str(""))
        };
        item.setEnabled(true);
        menu.addItem(&item);
    }

    fn act_with_target(menu: &NSMenu, mtm: MainThreadMarker, title: &str, action: Sel, target: *mut AnyObject) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(title), Some(action), &NSString::from_str(""))
        };
        item.setEnabled(true);
        unsafe { let _: () = objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setTarget: target]; }
        menu.addItem(&item);
    }

    fn act_with_target_and_tag(menu: &NSMenu, mtm: MainThreadMarker, title: &str, action: Sel, target: *mut AnyObject, launch_name: &str) {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(title), Some(action), &NSString::from_str(""))
        };
        item.setEnabled(true);
        unsafe {
            let _: () = objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setTarget: target];
            let ns = NSString::from_str(launch_name);
            let _: () = objc2::msg_send![&*item as *const NSMenuItem as *mut AnyObject, setRepresentedObject: &*ns];
        }
        menu.addItem(&item);
    }

    fn sep(menu: &NSMenu, mtm: MainThreadMarker) {
        menu.addItem(&NSMenuItem::separatorItem(mtm));
    }
}


