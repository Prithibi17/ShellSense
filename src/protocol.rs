use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "destructive")]
    Destructive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSuggestion {
    pub command: String,
    pub description: String,
    pub confidence: f32,
    pub risk: RiskLevel,
    pub warning: Option<String>,
    pub source: String, // "deterministic", "ai", "history"
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuggestTrigger {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "ghost")]
    Ghost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestRequest {
    pub input: String,
    pub cwd: Option<String>,
    pub shell: Option<String>,
    pub history: Option<Vec<String>>,
    pub trigger: SuggestTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestResponse {
    pub suggestions: Vec<CandidateSuggestion>,
    pub ai_available: bool,
    pub model: Option<String>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRequest {
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResponse {
    pub command: String,
    pub explanation: String,
    pub risk: RiskLevel,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRequest {
    pub input: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<String>,
    pub active_model: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub running: bool,
    pub ai_available: bool,
    pub provider: String,
    pub active_model: Option<String>,
    pub installed_models: Vec<String>,
    pub uptime_secs: u64,
    pub total_queries: u64,
    pub cache_size: usize,
    pub socket_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Suggest(SuggestRequest),
    Explain(ExplainRequest),
    Record(RecordRequest),
    Models(ModelsRequest),
    Status(StatusRequest),
    ClearCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Suggest(SuggestResponse),
    Explain(ExplainResponse),
    Record(RecordResponse),
    Models(ModelsResponse),
    Status(StatusResponse),
    CacheCleared,
    Error(String),
}
