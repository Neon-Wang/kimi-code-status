use super::UsageProvider;
use crate::types::{CredentialState, QuotaTier, ServiceQuota};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Types for Codex credential parsing ──

#[derive(serde::Deserialize)]
struct CodexAuthJson {
    auth_mode: Option<String>,
    tokens: Option<CodexTokens>,
    last_refresh: Option<String>,
}

#[derive(serde::Deserialize)]
struct CodexTokens {
    access_token: Option<String>,
    account_id: Option<String>,
}

// ── Types for Codex API response ──

#[derive(serde::Deserialize)]
struct CodexRateLimitWindow {
    used_percent: Option<f64>,
    limit_window_seconds: Option<i64>,
    reset_at: Option<i64>,
}

#[derive(serde::Deserialize)]
struct CodexRateLimit {
    primary_window: Option<CodexRateLimitWindow>,
    secondary_window: Option<CodexRateLimitWindow>,
}

#[derive(serde::Deserialize)]
struct CodexUsageResponse {
    rate_limit: Option<CodexRateLimit>,
}

// ── Helpers ──

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn window_seconds_to_tier_name(secs: i64) -> String {
    match secs {
        18000 => "five_hour".to_string(),
        604800 => "seven_day".to_string(),
        s => {
            let hours = s / 3600;
            if hours >= 24 {
                format!("{}_day", hours / 24)
            } else {
                format!("{}_hour", hours)
            }
        }
    }
}

fn unix_ts_to_iso(ts: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.to_rfc3339())
}

// ── Credential reading (adapted from cc-switch subscription.rs:470-580) ──

struct CodexCredentials {
    access_token: String,
    account_id: Option<String>,
    is_stale: bool,
}

/// Read Codex OAuth credentials. Tries:
/// 1. macOS Keychain (service: "Codex Auth")
/// 2. File: ~/.codex/auth.json
fn read_codex_credentials() -> Option<CodexCredentials> {
    // Try Keychain first (primary: Codex CLI stores credentials here)
    if let Some(creds) = read_from_keychain() {
        return Some(creds);
    }
    // Fallback to file
    read_from_file()
}

fn read_from_keychain() -> Option<CodexCredentials> {
    // Use security CLI since the keychain entry "Codex Auth" stores
    // arbitrary JSON that the security-framework crate may not find
    // with the generic password interface.
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Codex Auth", "-w"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8(output.stdout).ok()?;
    let json_str = json_str.trim();
    if json_str.is_empty() {
        return None;
    }

    parse_codex_json(json_str)
}

fn read_from_file() -> Option<CodexCredentials> {
    let home = dirs::home_dir()?;
    let auth_path = home.join(".codex").join("auth.json");

    if !auth_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&auth_path).ok()?;
    parse_codex_json(&content)
}

fn parse_codex_json(content: &str) -> Option<CodexCredentials> {
    let auth: CodexAuthJson = serde_json::from_str(content).ok()?;

    // Only OAuth mode has usage data
    if auth.auth_mode.as_deref() != Some("chatgpt") {
        return None;
    }

    let tokens = auth.tokens?;
    let access_token = tokens.access_token?;
    if access_token.is_empty() {
        return None;
    }

    // Check if token might be stale (last refresh > 8 days ago)
    let is_stale = auth.last_refresh.as_ref().is_some_and(|last_refresh| {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_refresh) {
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let age_secs = now_secs.saturating_sub(dt.timestamp() as u64);
            age_secs > 8 * 24 * 3600
        } else {
            false
        }
    });

    Some(CodexCredentials {
        access_token,
        account_id: tokens.account_id,
        is_stale,
    })
}

// ── Query function (adapted from cc-switch subscription.rs:643-732) ──

async fn query_codex(
    access_token: &str,
    account_id: Option<&str>,
) -> ServiceQuota {
    let client = reqwest::Client::new();

    let mut req = client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "codex-cli")
        .header("Accept", "application/json");

    if let Some(id) = account_id {
        req = req.header("ChatGPT-Account-Id", id);
    }

    let resp = match req.timeout(std::time::Duration::from_secs(10)).send().await {
        Ok(r) => r,
        Err(e) => {
            return ServiceQuota {
                service: "codex".into(),
                display_name: "Codex".into(),
                success: false,
                tiers: vec![],
                error: Some(format!("Network error: {e}")),
                queried_at: Some(now_millis()),
                credential_valid: true,
            };
        }
    };

    let status = resp.status();

    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return ServiceQuota {
            service: "codex".into(),
            display_name: "Codex".into(),
            success: false,
            tiers: vec![],
            error: Some("Session expired. Re-login with Codex CLI.".into()),
            queried_at: Some(now_millis()),
            credential_valid: false,
        };
    }

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return ServiceQuota {
            service: "codex".into(),
            display_name: "Codex".into(),
            success: false,
            tiers: vec![],
            error: Some(format!("API error (HTTP {status}): {body}")),
            queried_at: Some(now_millis()),
            credential_valid: true,
        };
    }

    let body: CodexUsageResponse = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return ServiceQuota {
                service: "codex".into(),
                display_name: "Codex".into(),
                success: false,
                tiers: vec![],
                error: Some(format!("Failed to parse response: {e}")),
                queried_at: Some(now_millis()),
                credential_valid: true,
            };
        }
    };

    let mut tiers = Vec::new();

    if let Some(rate_limit) = body.rate_limit {
        for window in [rate_limit.primary_window, rate_limit.secondary_window]
            .into_iter()
            .flatten()
        {
            if let Some(used) = window.used_percent {
                tiers.push(QuotaTier {
                    name: window
                        .limit_window_seconds
                        .map(window_seconds_to_tier_name)
                        .unwrap_or_else(|| "unknown".to_string()),
                    utilization: used,
                    resets_at: window.reset_at.and_then(unix_ts_to_iso),
                });
            }
        }
    }

    ServiceQuota {
        service: "codex".into(),
        display_name: "Codex".into(),
        success: true,
        tiers,
        error: None,
        queried_at: Some(now_millis()),
        credential_valid: true,
    }
}

// ── Codex Provider ──

pub struct CodexProvider;

impl CodexProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UsageProvider for CodexProvider {
    fn service_id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    fn needs_credentials(&self) -> bool {
        read_codex_credentials().is_none()
    }

    fn credential_state(&self) -> CredentialState {
        match read_codex_credentials() {
            Some(creds) if creds.is_stale => {
                CredentialState::Expired("Token may be stale (>8 days)".into())
            }
            Some(_) => CredentialState::Valid,
            None => CredentialState::Missing,
        }
    }

    async fn query(&self) -> ServiceQuota {
        let creds = match read_codex_credentials() {
            Some(c) => c,
            None => {
                return ServiceQuota {
                    service: "codex".into(),
                    display_name: "Codex".into(),
                    success: false,
                    tiers: vec![],
                    error: Some("Codex OAuth credentials not found. Login with Codex CLI first.".into()),
                    queried_at: None,
                    credential_valid: false,
                };
            }
        };

        let mut quota = query_codex(&creds.access_token, creds.account_id.as_deref()).await;

        // If token is stale, note it even if the API call succeeded
        if creds.is_stale && quota.success {
            quota.error = Some("Token may be stale (>8 days since refresh)".into());
        }

        quota
    }
}
