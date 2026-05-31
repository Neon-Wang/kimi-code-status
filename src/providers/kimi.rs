use super::UsageProvider;
use crate::keychain;
use crate::types::{CredentialState, QuotaTier, ServiceQuota};
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Helpers (adapted from cc-switch services/coding_plan.rs) ──

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn parse_f64(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

fn extract_reset_time(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    if let Some(n) = value.as_i64() {
        let ms = if n < 1_000_000_000_000 { n * 1000 } else { n };
        let secs = ms / 1000;
        let nsecs = ((ms % 1000) * 1_000_000) as u32;
        return chrono::DateTime::from_timestamp(secs, nsecs)
            .map(|dt| dt.to_rfc3339());
    }
    None
}

// ── Kimi Provider ──

pub struct KimiProvider;

impl KimiProvider {
    pub fn new() -> Self {
        Self
    }

    fn load_api_key() -> Result<Option<String>, String> {
        keychain::load_api_key(keychain::KIMI_SERVICE, keychain::KIMI_ACCOUNT)
    }

    pub fn store_api_key(key: &str) -> Result<(), String> {
        keychain::store_api_key(keychain::KIMI_SERVICE, keychain::KIMI_ACCOUNT, key)
    }
}

#[async_trait]
impl UsageProvider for KimiProvider {
    fn service_id(&self) -> &'static str {
        "kimi"
    }

    fn display_name(&self) -> &'static str {
        "Kimi Code"
    }

    fn needs_credentials(&self) -> bool {
        Self::load_api_key().ok().flatten().is_none()
    }

    fn credential_state(&self) -> CredentialState {
        match Self::load_api_key() {
            Ok(Some(_)) => CredentialState::Valid,
            Ok(None) => CredentialState::Missing,
            Err(e) => CredentialState::Expired(e),
        }
    }

    async fn query(&self) -> ServiceQuota {
        let api_key = match Self::load_api_key() {
            Ok(Some(key)) => key,
            Ok(None) => {
                return ServiceQuota {
                    service: "kimi".into(),
                    display_name: "Kimi Code".into(),
                    success: false,
                    tiers: vec![],
                    error: Some("API key not configured".into()),
                    queried_at: None,
                    credential_valid: false,
                };
            }
            Err(e) => {
                return ServiceQuota {
                    service: "kimi".into(),
                    display_name: "Kimi Code".into(),
                    success: false,
                    tiers: vec![],
                    error: Some(e),
                    queried_at: None,
                    credential_valid: false,
                };
            }
        };

        query_kimi(&api_key).await
    }
}

// ── Core query function (adapted from cc-switch coding_plan.rs:88-182) ──

async fn query_kimi(api_key: &str) -> ServiceQuota {
    let client = reqwest::Client::new();

    let resp = match client
        .get("https://api.kimi.com/coding/v1/usages")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return ServiceQuota {
                service: "kimi".into(),
                display_name: "Kimi Code".into(),
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
            service: "kimi".into(),
            display_name: "Kimi Code".into(),
            success: false,
            tiers: vec![],
            error: Some("Invalid API key".into()),
            queried_at: Some(now_millis()),
            credential_valid: false,
        };
    }

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return ServiceQuota {
            service: "kimi".into(),
            display_name: "Kimi Code".into(),
            success: false,
            tiers: vec![],
            error: Some(format!("API error (HTTP {status}): {body}")),
            queried_at: Some(now_millis()),
            credential_valid: true,
        };
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return ServiceQuota {
                service: "kimi".into(),
                display_name: "Kimi Code".into(),
                success: false,
                tiers: vec![],
                error: Some(format!("Failed to parse response: {e}")),
                queried_at: Some(now_millis()),
                credential_valid: true,
            };
        }
    };

    let mut tiers = Vec::new();

    // 5 小时窗口
    if let Some(limits) = body.get("limits").and_then(|v| v.as_array()) {
        for limit_item in limits {
            if let Some(detail) = limit_item.get("detail") {
                let limit = detail.get("limit").and_then(parse_f64).unwrap_or(1.0);
                let remaining = detail.get("remaining").and_then(parse_f64).unwrap_or(0.0);
                let resets_at = detail.get("resetTime").and_then(extract_reset_time);

                let used = (limit - remaining).max(0.0);
                let utilization = if limit > 0.0 {
                    (used / limit) * 100.0
                } else {
                    0.0
                };
                tiers.push(QuotaTier {
                    name: "five_hour".to_string(),
                    utilization,
                    resets_at,
                });
            }
        }
    }

    // 周限额
    if let Some(usage) = body.get("usage") {
        let limit = usage.get("limit").and_then(parse_f64).unwrap_or(1.0);
        let remaining = usage.get("remaining").and_then(parse_f64).unwrap_or(0.0);
        let resets_at = usage.get("resetTime").and_then(extract_reset_time);

        let used = (limit - remaining).max(0.0);
        let utilization = if limit > 0.0 {
            (used / limit) * 100.0
        } else {
            0.0
        };
        tiers.push(QuotaTier {
            name: "weekly_limit".to_string(),
            utilization,
            resets_at,
        });
    }

    ServiceQuota {
        service: "kimi".into(),
        display_name: "Kimi Code".into(),
        success: true,
        tiers,
        error: None,
        queried_at: Some(now_millis()),
        credential_valid: true,
    }
}
