use crate::types::{ToolInfo, ToolType};

// ── Known AI tools registry ──

struct ToolDef {
    id: &'static str,
    name: &'static str,
    tool_type: ToolType,
    /// Directory to check under ~/
    home_dir: Option<&'static str>,
    /// Binary name to check via `which`
    binary: Option<&'static str>,
    /// Application path under /Applications
    app_path: Option<&'static str>,
}

const KNOWN_TOOLS: &[ToolDef] = &[
    ToolDef {
        id: "claude_code",
        name: "Claude Code",
        tool_type: ToolType::CLI,
        home_dir: Some(".claude"),
        binary: Some("claude"),
        app_path: None,
    },
    ToolDef {
        id: "codex_cli",
        name: "Codex CLI",
        tool_type: ToolType::CLI,
        home_dir: Some(".codex"),
        binary: Some("codex"),
        app_path: None,
    },
    ToolDef {
        id: "gemini_cli",
        name: "Gemini CLI",
        tool_type: ToolType::CLI,
        home_dir: Some(".gemini"),
        binary: Some("gemini"),
        app_path: None,
    },
    ToolDef {
        id: "opencode",
        name: "OpenCode",
        tool_type: ToolType::CLI,
        home_dir: Some(".opencode"),
        binary: Some("opencode"),
        app_path: None,
    },
    ToolDef {
        id: "openclaw",
        name: "OpenClaw",
        tool_type: ToolType::CLI,
        home_dir: Some(".openclaw"),
        binary: Some("openclaw"),
        app_path: None,
    },
    ToolDef {
        id: "hermes",
        name: "Hermes",
        tool_type: ToolType::CLI,
        home_dir: Some(".hermes"),
        binary: None,
        app_path: None,
    },
    ToolDef {
        id: "cursor",
        name: "Cursor",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Cursor.app"),
    },
    ToolDef {
        id: "windsurf",
        name: "Windsurf",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Windsurf.app"),
    },
    ToolDef {
        id: "vscode",
        name: "VS Code",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Visual Studio Code.app"),
    },
    ToolDef {
        id: "code_oss",
        name: "VS Code (OSS)",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Visual Studio Code - OSS.app"),
    },
    ToolDef {
        id: "augment_code",
        name: "Augment Code",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Augment Code.app"),
    },
    ToolDef {
        id: "trae",
        name: "Trae",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Trae.app"),
    },
    ToolDef {
        id: "trae_cn",
        name: "Trae CN",
        tool_type: ToolType::IDE,
        home_dir: None,
        binary: None,
        app_path: Some("/Applications/Trae CN.app"),
    },
    ToolDef {
        id: "qoder",
        name: "Qoder",
        tool_type: ToolType::CLI,
        home_dir: Some(".qoder"),
        binary: Some("qoder"),
        app_path: None,
    },
    ToolDef {
        id: "cc_switch",
        name: "CC Switch",
        tool_type: ToolType::CLI,
        home_dir: Some(".cc-switch"),
        binary: None,
        app_path: Some("/Applications/CC Switch.app"),
    },
];

fn home_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir()
}

/// Check if a binary exists on PATH via `which`.
fn check_binary(name: &str) -> Option<String> {
    let output = std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

/// Check if a directory exists under home.
fn check_home_dir(subdir: &str) -> Option<String> {
    let path = home_dir()?.join(subdir);
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

/// Check if an .app bundle exists.
fn check_app(app_path: &str) -> Option<String> {
    let path = std::path::Path::new(app_path);
    if path.exists() {
        Some(app_path.to_string())
    } else {
        None
    }
}

/// Scan for all known AI tools and return their detection results.
pub fn scan_tools() -> Vec<ToolInfo> {
    KNOWN_TOOLS
        .iter()
        .map(|def| {
            let mut install_path = None;

            // Try home dir detection
            if let Some(subdir) = def.home_dir {
                install_path = check_home_dir(subdir);
            }

            // Try binary detection
            if install_path.is_none() {
                if let Some(bin) = def.binary {
                    install_path = check_binary(bin);
                }
            }

            // Try .app detection
            if install_path.is_none() {
                if let Some(app) = def.app_path {
                    install_path = check_app(app);
                }
            }

            ToolInfo {
                id: def.id.to_string(),
                name: def.name.to_string(),
                tool_type: def.tool_type.clone(),
                installed: install_path.is_some(),
                install_path,
            }
        })
        .collect()
}
