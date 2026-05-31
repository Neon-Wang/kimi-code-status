pub mod codex;
pub mod kimi;

use crate::types::{CredentialState, ServiceQuota};
use async_trait::async_trait;

#[async_trait]
pub trait UsageProvider: Send + Sync {
    /// Unique service identifier: "kimi" / "codex"
    fn service_id(&self) -> &'static str;

    /// Display name for menus: "Kimi Code" / "Codex"
    fn display_name(&self) -> &'static str;

    /// Query usage from the service API.
    async fn query(&self) -> ServiceQuota;

    /// Whether this provider needs credentials configured.
    fn needs_credentials(&self) -> bool;

    /// Current credential state.
    fn credential_state(&self) -> CredentialState;
}
