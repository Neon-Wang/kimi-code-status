use crate::types::{ToolInfo, ToolType};

struct ToolDef {
    id: &'static str,
    name: &'static str,
    tool_type: ToolType,
    home_dir: Option<&'static str>,
    binary: Option<&'static str>,
    app_path: Option<&'static str>,
    launch_as: Option<&'static str>,
}

const KNOWN_TOOLS: &[ToolDef] = &[
    // ── JetBrains IDEs ──
    ToolDef { id: "intellij",        name: "IntelliJ IDEA",       tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/IntelliJ IDEA.app"),                      launch_as: Some("IntelliJ IDEA") },
    ToolDef { id: "intellij_ce",     name: "IntelliJ IDEA CE",    tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/IntelliJ IDEA CE.app"),                   launch_as: Some("IntelliJ IDEA CE") },
    ToolDef { id: "pycharm",         name: "PyCharm",             tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/PyCharm.app"),                           launch_as: Some("PyCharm") },
    ToolDef { id: "pycharm_ce",      name: "PyCharm CE",          tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/PyCharm CE.app"),                        launch_as: Some("PyCharm CE") },
    ToolDef { id: "webstorm",        name: "WebStorm",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/WebStorm.app"),                          launch_as: Some("WebStorm") },
    ToolDef { id: "goland",          name: "GoLand",              tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/GoLand.app"),                            launch_as: Some("GoLand") },
    ToolDef { id: "clion",           name: "CLion",               tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/CLion.app"),                             launch_as: Some("CLion") },
    ToolDef { id: "rustrover",       name: "RustRover",           tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/RustRover.app"),                         launch_as: Some("RustRover") },
    ToolDef { id: "fleet",           name: "Fleet",               tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Fleet.app"),                             launch_as: Some("Fleet") },
    ToolDef { id: "rubymine",        name: "RubyMine",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/RubyMine.app"),                          launch_as: Some("RubyMine") },
    ToolDef { id: "phpstorm",        name: "PhpStorm",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/PhpStorm.app"),                          launch_as: Some("PhpStorm") },
    ToolDef { id: "datagrip",        name: "DataGrip",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/DataGrip.app"),                          launch_as: Some("DataGrip") },
    ToolDef { id: "rider",           name: "Rider",               tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Rider.app"),                             launch_as: Some("Rider") },
    ToolDef { id: "android_studio",  name: "Android Studio",      tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Android Studio.app"),                    launch_as: Some("Android Studio") },
    // ── Apple ──
    ToolDef { id: "xcode",           name: "Xcode",               tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Xcode.app"),                             launch_as: Some("Xcode") },
    // ── VS Code Family ──
    ToolDef { id: "vscode",          name: "VS Code",             tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Visual Studio Code.app"),                launch_as: Some("Visual Studio Code") },
    ToolDef { id: "code_oss",        name: "VS Code OSS",         tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Visual Studio Code - OSS.app"),           launch_as: Some("Visual Studio Code - OSS") },
    ToolDef { id: "vscode_insiders", name: "VS Code Insiders",    tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Visual Studio Code - Insiders.app"),      launch_as: Some("Visual Studio Code - Insiders") },
    // ── AI IDEs ──
    ToolDef { id: "cursor",          name: "Cursor",              tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Cursor.app"),                            launch_as: Some("Cursor") },
    ToolDef { id: "windsurf",        name: "Windsurf",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Windsurf.app"),                          launch_as: Some("Windsurf") },
    ToolDef { id: "trae",            name: "Trae",                tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Trae.app"),                              launch_as: Some("Trae") },
    ToolDef { id: "trae_cn",         name: "Trae CN",             tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Trae CN.app"),                           launch_as: Some("Trae CN") },
    ToolDef { id: "augment",         name: "Augment Code",        tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Augment Code.app"),                      launch_as: Some("Augment Code") },
    ToolDef { id: "solo",            name: "Solo",                tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Solo.app"),                              launch_as: Some("Solo") },
    ToolDef { id: "t3_code",         name: "T3 Code",             tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/T3 Code.app"),                           launch_as: Some("T3 Code") },
    ToolDef { id: "copilot",         name: "GitHub Copilot",      tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/GitHub Copilot.app"),                    launch_as: Some("GitHub Copilot") },
    ToolDef { id: "cc_switch",       name: "CC Switch",           tool_type: ToolType::IDE, home_dir: Some(".cc-switch"), binary: None, app_path: Some("/Applications/CC Switch.app"),               launch_as: Some("CC Switch") },
    ToolDef { id: "kimi",            name: "Kimi",                tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Kimi.app"),                              launch_as: Some("Kimi") },
    ToolDef { id: "codex_app",       name: "Codex",               tool_type: ToolType::IDE, home_dir: Some(".codex"), binary: None, app_path: Some("/Applications/Codex.app"),                        launch_as: Some("Codex") },
    // ── Terminals ──
    ToolDef { id: "ghostty",         name: "Ghostty",             tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Ghostty.app"),                           launch_as: Some("Ghostty") },
    ToolDef { id: "warp",            name: "Warp",                tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Warp.app"),                              launch_as: Some("Warp") },
    ToolDef { id: "iterm2",          name: "iTerm2",              tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/iTerm.app"),                             launch_as: Some("iTerm") },
    ToolDef { id: "terminal",        name: "Terminal",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/System/Applications/Utilities/Terminal.app"),         launch_as: Some("Terminal") },
    // ── Dev tools ──
    ToolDef { id: "docker",          name: "Docker Desktop",      tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Docker.app"),                            launch_as: Some("Docker") },
    ToolDef { id: "orbstack",        name: "OrbStack",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/OrbStack.app"),                          launch_as: Some("OrbStack") },
    ToolDef { id: "sublime_text",    name: "Sublime Text",        tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Sublime Text.app"),                      launch_as: Some("Sublime Text") },
    ToolDef { id: "nova",            name: "Nova",                tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Nova.app"),                              launch_as: Some("Nova") },
    ToolDef { id: "zed",             name: "Zed",                 tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Zed.app"),                               launch_as: Some("Zed") },
    ToolDef { id: "obsidian",        name: "Obsidian",            tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Obsidian.app"),                          launch_as: Some("Obsidian") },
    ToolDef { id: "raycast",         name: "Raycast",             tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Raycast.app"),                           launch_as: Some("Raycast") },

    // ── CLI tools (click → pick folder → Ghostty cd + run) ──
    ToolDef { id: "claude_code",     name: "Claude Code",   tool_type: ToolType::CLI, home_dir: Some(".claude"),   binary: Some("claude"),   app_path: None, launch_as: None },
    ToolDef { id: "codex_cli",       name: "Codex CLI",     tool_type: ToolType::CLI, home_dir: Some(".codex"),    binary: Some("codex"),    app_path: None, launch_as: None },
    ToolDef { id: "gemini_cli",      name: "Gemini CLI",    tool_type: ToolType::CLI, home_dir: Some(".gemini"),   binary: Some("gemini"),   app_path: None, launch_as: None },
    ToolDef { id: "opencode",        name: "OpenCode",      tool_type: ToolType::CLI, home_dir: Some(".opencode"), binary: Some("opencode"), app_path: None, launch_as: None },
    ToolDef { id: "openclaw",        name: "OpenClaw",      tool_type: ToolType::CLI, home_dir: Some(".openclaw"), binary: Some("openclaw"), app_path: None, launch_as: None },
    ToolDef { id: "hermes",          name: "Hermes",        tool_type: ToolType::CLI, home_dir: Some(".hermes"),   binary: None,            app_path: None, launch_as: None },
    ToolDef { id: "qoder",           name: "Qoder",         tool_type: ToolType::CLI, home_dir: Some(".qoder"),    binary: Some("qoder"),   app_path: None, launch_as: None },
    ToolDef { id: "aider",           name: "Aider",         tool_type: ToolType::CLI, home_dir: None,              binary: Some("aider"),   app_path: None, launch_as: None },
    ToolDef { id: "cursor_cli",      name: "Cursor CLI",    tool_type: ToolType::CLI, home_dir: None,              binary: Some("cursor"),  app_path: None, launch_as: None },
    ToolDef { id: "windsurf_cli",    name: "Windsurf CLI",  tool_type: ToolType::CLI, home_dir: None,              binary: Some("windsurf"), app_path: None, launch_as: None },
];

fn check_binary(name: &str) -> Option<String> {
    std::process::Command::new("which").arg(name).output().ok().and_then(|o| {
        if o.status.success() { String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) } else { None }
    })
}

fn check_home_dir(subdir: &str) -> Option<String> {
    let path = dirs::home_dir()?.join(subdir);
    if path.exists() { Some(path.to_string_lossy().to_string()) } else { None }
}

fn check_app(app_path: &str) -> Option<String> {
    if std::path::Path::new(app_path).exists() { Some(app_path.to_string()) } else { None }
}

pub fn scan_tools() -> Vec<ToolInfo> {
    KNOWN_TOOLS.iter().filter_map(|def| {
        let mut install_path = None;
        if let Some(d) = def.home_dir { install_path = check_home_dir(d); }
        if install_path.is_none() { if let Some(b) = def.binary { install_path = check_binary(b); } }
        if install_path.is_none() { if let Some(a) = def.app_path { install_path = check_app(a); } }

        let install_path = install_path?;

        Some(ToolInfo {
            id: def.id.to_string(),
            name: def.name.to_string(),
            tool_type: def.tool_type.clone(),
            installed: true,
            install_path: Some(install_path),
            launch_as: def.launch_as.map(|s| s.to_string()),
        })
    }).collect()
}
