use crate::types::QuotaTier;

pub const UTIL_DANGER_PCT: f64 = 90.0;
pub const UTIL_WARN_PCT: f64 = 70.0;

/// Color emoji for utilization percentage.
/// 🟢 < 70%, 🟠 70-90%, 🔴 >= 90%
pub fn emoji_for_utilization(pct: f64) -> &'static str {
    if pct >= UTIL_DANGER_PCT {
        "\u{1F534}" // 🔴
    } else if pct >= UTIL_WARN_PCT {
        "\u{1F7E0}" // 🟠
    } else {
        "\u{1F7E2}" // 🟢
    }
}

/// Format tiers into a compact summary string like "🟢 h12% w45%".
/// Handles Kimi/Codex-style tiers: five_hour → "h", weekly_limit → "w".
pub fn format_summary(tiers: &[QuotaTier]) -> Option<String> {
    if tiers.is_empty() {
        return None;
    }

    let mut parts: Vec<(&'static str, f64)> = Vec::new();
    for t in tiers {
        let label = match t.name.as_str() {
            "five_hour" => "h",
            "weekly_limit" => "w",
            "seven_day" => "w",
            _ => continue,
        };
        parts.push((label, t.utilization));
    }

    if parts.is_empty() {
        return None;
    }

    let worst = parts
        .iter()
        .map(|(_, u)| *u)
        .fold(f64::NEG_INFINITY, f64::max);

    if !worst.is_finite() {
        return None;
    }

    let emoji = emoji_for_utilization(worst);
    let body = parts
        .iter()
        .map(|(label, u)| format!("{label}{}%", u.round() as i64))
        .collect::<Vec<_>>()
        .join(" ");

    Some(format!("{emoji} {body}"))
}

/// Short status: just the emoji for the worst tier.
pub fn status_emoji(tiers: &[QuotaTier]) -> &'static str {
    let worst = tiers
        .iter()
        .map(|t| t.utilization)
        .fold(f64::NEG_INFINITY, f64::max);
    if !worst.is_finite() {
        return "\u{26AA}"; // ⚪ unknown
    }
    emoji_for_utilization(worst)
}
