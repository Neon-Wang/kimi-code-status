use crate::types::{ToolInfo, ToolType};

struct ToolDef {
    id: &'static str,
    name: &'static str,
    tool_type: ToolType,
    home_dir: Option<&'static str>,
    binary: Option<&'static str>,
    app_path: Option<&'static str>,
    /// How to launch: if set, use `open -a <launch>`; else use the detected path
    launch_as: Option<&'static str>,
}

const KNOWN_TOOLS: &[ToolDef] = &[
    // IDEs — launch via `open -a <name>`
    ToolDef { id: "cursor", name: "Cursor", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Cursor.app"), launch_as: Some("Cursor") },
    ToolDef { id: "windsurf", name: "Windsurf", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Windsurf.app"), launch_as: Some("Windsurf") },
    ToolDef { id: "vscode", name: "VS Code", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Visual Studio Code.app"), launch_as: Some("Visual Studio Code") },
    ToolDef { id: "code_oss", name: "VS Code OSS", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Visual Studio Code - OSS.app"), launch_as: Some("Visual Studio Code - OSS") },
    ToolDef { id: "augment", name: "Augment Code", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Augment Code.app"), launch_as: Some("Augment Code") },
    ToolDef { id: "trae", name: "Trae", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Trae.app"), launch_as: Some("Trae") },
    ToolDef { id: "trae_cn", name: "Trae CN", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Trae CN.app"), launch_as: Some("Trae CN") },
    ToolDef { id: "solo", name: "Solo", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/Solo.app"), launch_as: Some("Solo") },
    ToolDef { id: "t3_code", name: "T3 Code", tool_type: ToolType::IDE, home_dir: None, binary: None, app_path: Some("/Applications/T3 Code.app"), launch_as: Some("T3 Code") },
    // CLI tools — open via terminal or jump to directory
    ToolDef { id: "claude_code", name: "Claude Code", tool_type: ToolType::CLI, home_dir: Some(".claude"), binary: Some("claude"), app_path: None, launch_as: None },
    ToolDef { id: "codex_cli", name: "Codex CLI", tool_type: ToolType::CLI, home_dir: Some(".codex"), binary: Some("codex"), app_path: None, launch_as: None },
    ToolDef { id: "gemini_cli", name: "Gemini CLI", tool_type: ToolType::CLI, home_dir: Some(".gemini"), binary: Some("gemini"), app_path: None, launch_as: None },
    ToolDef { id: "opencode", name: "OpenCode", tool_type: ToolType::CLI, home_dir: Some(".opencode"), binary: Some("opencode"), app_path: None, launch_as: None },
    ToolDef { id: "openclaw", name: "OpenClaw", tool_type: ToolType::CLI, home_dir: Some(".openclaw"), binary: Some("openclaw"), app_path: None, launch_as: None },
    ToolDef { id: "hermes", name: "Hermes", tool_type: ToolType::CLI, home_dir: Some(".hermes"), binary: None, app_path: None, launch_as: None },
    ToolDef { id: "qoder", name: "Qoder", tool_type: ToolType::CLI, home_dir: Some(".qoder"), binary: Some("qoder"), app_path: None, launch_as: None },
    ToolDef { id: "cc_switch", name: "CC Switch", tool_type: ToolType::CLI, home_dir: Some(".cc-switch"), binary: None, app_path: Some("/Applications/CC Switch.app"), launch_as: Some("CC Switch") },
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
    KNOWN_TOOLS.iter().map(|def| {
        let mut install_path = None;
        if let Some(d) = def.home_dir { install_path = check_home_dir(d); }
        if install_path.is_none() { if let Some(b) = def.binary { install_path = check_binary(b); } }
        if install_path.is_none() { if let Some(a) = def.app_path { install_path = check_app(a); } }

        ToolInfo {
            id: def.id.to_string(),
            name: def.name.to_string(),
            tool_type: def.tool_type.clone(),
            installed: install_path.is_some(),
            install_path,
            launch_as: def.launch_as.map(|s| s.to_string()),
        }
    }).collect()
}
