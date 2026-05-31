use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

/// Store an API key in the macOS Keychain.
pub fn store_api_key(service: &str, account: &str, key: &str) -> Result<(), String> {
    set_generic_password(service, account, key.as_bytes())
        .map_err(|e| format!("Failed to store API key in Keychain: {e}"))
}

/// Load an API key from the macOS Keychain.
pub fn load_api_key(service: &str, account: &str) -> Result<Option<String>, String> {
    match get_generic_password(service, account) {
        Ok(data) => {
            String::from_utf8(data).map(Some).map_err(|e| {
                format!("Keychain data is not valid UTF-8: {e}")
            })
        }
        Err(e) => {
            // errSecItemNotFound means the key doesn't exist yet
            if e.to_string().contains("-25300") || e.code() == -25300 {
                return Ok(None);
            }
            Err(format!("Failed to read from Keychain: {e}"))
        }
    }
}

/// Delete an API key from the macOS Keychain.
pub fn delete_api_key(service: &str, account: &str) -> Result<(), String> {
    delete_generic_password(service, account)
        .map_err(|e| format!("Failed to delete from Keychain: {e}"))
}

// ── App-specific constants ──

pub const KIMI_SERVICE: &str = "Kimi Code Status";
pub const KIMI_ACCOUNT: &str = "kimi-api-key";

// Codex Keychain entry (created by Codex CLI, read-only for us)
pub const CODEX_SERVICE: &str = "Codex Auth";
pub const CODEX_ACCOUNT: &str = "codex-credentials";
