pub mod ollama;

use crate::context::SystemContext;
use crate::protocol::CandidateSuggestion;
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn suggest(&self, input: &str, ctx: &SystemContext) -> Result<Vec<CandidateSuggestion>, String>;
    async fn explain(&self, command: &str) -> Result<String, String>;
    async fn list_models(&self) -> Result<Vec<String>, String>;
    fn name(&self) -> &'static str;
}
