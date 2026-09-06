pub mod cache;
pub mod config;
pub mod context;
pub mod deterministic;
pub mod fuzzy;
pub mod learning;
pub mod protocol;
pub mod provider;
pub mod safety;

use cache::SuggestionCache;
use config::Config;
use context::SystemContext;
use deterministic::match_deterministic;
use learning::LearningEngine;
use protocol::{CandidateSuggestion, ExplainResponse, Request, Response, SuggestRequest, SuggestResponse};
use provider::ollama::OllamaProvider;
use provider::AIProvider;
use safety::check_safety;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

pub struct AssistantEngine {
    pub config: Config,
    pub provider: Arc<dyn AIProvider>,
    pub cache: Arc<SuggestionCache>,
    pub learning: Arc<LearningEngine>,
    pub start_time: Instant,
}

impl AssistantEngine {
    pub fn new(config: Config) -> Self {
        let provider: Arc<dyn AIProvider> = Arc::new(OllamaProvider::new(
            config.ai.endpoint.clone(),
            config.ai.model.clone(),
            config.ai.timeout_ms,
        ));
        let cache = Arc::new(SuggestionCache::new(600, 500));
        let learning = Arc::new(LearningEngine::new());

        Self {
            config,
            provider,
            cache,
            learning,
            start_time: Instant::now(),
        }
    }

    pub async fn handle_request(&self, req: Request) -> Response {
        match req {
            Request::Suggest(suggest_req) => {
                let resp = self.suggest(suggest_req).await;
                Response::Suggest(resp)
            }
            Request::Explain(explain_req) => {
                let resp = self.explain(&explain_req.command).await;
                Response::Explain(resp)
            }
            Request::Record(rec_req) => {
                let res = self.learning.record_accepted(&rec_req.input, &rec_req.command);
                Response::Record(protocol::RecordResponse {
                    success: res.is_ok(),
                })
            }
            Request::Models(_) => {
                let models = self.provider.list_models().await.unwrap_or_default();
                let active = if let Some(ollama) = self.provider.as_any_ollama() {
                    ollama.select_active_model().await.ok()
                } else {
                    None
                };
                Response::Models(protocol::ModelsResponse {
                    models,
                    active_model: active,
                    provider: self.provider.name().to_string(),
                })
            }
            Request::Status(_) => {
                let models = self.provider.list_models().await.unwrap_or_default();
                let ai_available = !models.is_empty();
                let active = if let Some(ollama) = self.provider.as_any_ollama() {
                    ollama.select_active_model().await.ok()
                } else {
                    None
                };

                Response::Status(protocol::StatusResponse {
                    running: true,
                    ai_available,
                    provider: self.provider.name().to_string(),
                    active_model: active,
                    installed_models: models,
                    uptime_secs: self.start_time.elapsed().as_secs(),
                    total_queries: 0,
                    cache_size: self.cache.len(),
                    socket_path: Config::socket_path().display().to_string(),
                })
            }
            Request::ClearCache => {
                self.cache.clear();
                Response::CacheCleared
            }
        }
    }

    pub async fn suggest(&self, req: SuggestRequest) -> SuggestResponse {
        let start = Instant::now();
        let cwd_path = req.cwd.as_ref().map(|s| Path::new(s.as_str()));
        let ctx = SystemContext::gather(cwd_path, req.shell.as_deref(), req.history.as_deref());

        // 1. Check in-memory cache
        if let Some(cached) = self.cache.get(&req.input, &ctx.cwd) {
            return SuggestResponse {
                suggestions: cached,
                ai_available: true,
                model: None,
                latency_ms: start.elapsed().as_millis() as u64,
            };
        }

        let mut candidates: Vec<CandidateSuggestion> = Vec::new();

        // 2. Check local learning history
        let history_matches = self.learning.lookup(&req.input);
        candidates.extend(history_matches);

        // 3. Match deterministic / local CachyOS rules
        let det_matches = match_deterministic(&req.input, &ctx);
        let has_strong_det = det_matches.iter().any(|s| s.confidence >= 0.95);
        candidates.extend(det_matches);

        // 4. Query AI intent engine if needed and enabled
        // IMPORTANT: Skip AI for Ghost (inline keystroke) triggers — per-keystroke LLM inference
        // is too slow and prone to hallucinating destructive commands on partial inputs like "rm l".
        let mut ai_available = false;
        let mut model_used = None;

        let is_ghost = req.trigger == protocol::SuggestTrigger::Ghost;

        if !is_ghost && self.config.general.ai_enabled && (!has_strong_det || candidates.len() < self.config.general.max_suggestions) {
            match self.provider.suggest(&req.input, &ctx).await {
                Ok(ai_sugs) => {
                    ai_available = true;
                    if let Some(ollama) = self.provider.as_any_ollama() {
                        model_used = ollama.select_active_model().await.ok();
                    }
                    candidates.extend(ai_sugs);
                }
                Err(_err) => {
                    // AI is unavailable (e.g. Ollama offline or zero models), continue gracefully
                    ai_available = false;
                }
            }
        }

        // 5. Deduplicate and validate safety
        let mut seen = HashSet::new();
        let mut final_suggestions = Vec::new();

        // Sort primarily by confidence descending
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        for cand in candidates {
            let safety = check_safety(&cand.command);
            let normalized = safety.normalized_command;

            if !seen.contains(&normalized) {
                seen.insert(normalized.clone());
                final_suggestions.push(CandidateSuggestion {
                    command: normalized,
                    description: cand.description,
                    confidence: cand.confidence - safety.confidence_penalty,
                    risk: safety.risk,
                    warning: safety.warning.or(cand.warning),
                    source: cand.source,
                    category: cand.category,
                });
            }

            if final_suggestions.len() >= self.config.general.max_suggestions {
                break;
            }
        }

        // 6. Cache and return
        if !final_suggestions.is_empty() {
            self.cache.insert(&req.input, &ctx.cwd, final_suggestions.clone());
        }

        SuggestResponse {
            suggestions: final_suggestions,
            ai_available,
            model: model_used,
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub async fn explain(&self, command: &str) -> ExplainResponse {
        let safety = check_safety(command);

        // Try AI explanation first
        let explanation = if self.config.general.ai_enabled {
            match self.provider.explain(command).await {
                Ok(exp) if !exp.is_empty() => exp,
                _ => generate_fallback_explanation(command, &safety),
            }
        } else {
            generate_fallback_explanation(command, &safety)
        };

        ExplainResponse {
            command: safety.normalized_command,
            explanation,
            risk: safety.risk,
            warning: safety.warning,
        }
    }
}

fn generate_fallback_explanation(command: &str, safety: &safety::SafetyReport) -> String {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    let mut idx = 0;
    while idx < tokens.len() && (tokens[idx] == "sudo" || tokens[idx] == "env" || tokens[idx].starts_with('-')) {
        idx += 1;
    }
    let bin = if idx < tokens.len() { tokens[idx] } else { tokens.first().copied().unwrap_or(command) };

    match bin {
        "pacman" => "Arch Linux package manager. Manages official packages and system updates.".to_string(),
        "paru" => "Pacman-wrapping AUR helper. Manages official Arch repositories and user-submitted AUR packages.".to_string(),
        "systemctl" => "systemd service and unit management utility.".to_string(),
        "nvidia-smi" => "NVIDIA System Management Interface. Monitors GPU utilization, temperature, and processes.".to_string(),
        "wpctl" => "WirePlumber control tool for PipeWire audio routing and volume.".to_string(),
        "hyprctl" => "Hyprland Wayland compositor control tool.".to_string(),
        "git" => "Distributed version control system.".to_string(),
        "ss" => "Socket statistics utility for inspecting open ports and network connections.".to_string(),
        _ => {
            if let Some(ref warn) = safety.warning {
                format!("Command '{}' ({})", command, warn)
            } else {
                format!("Linux command: '{}'", command)
            }
        }
    }
}

// Extension to allow querying Ollama specific helpers if available
impl dyn AIProvider {
    pub fn as_any_ollama(&self) -> Option<&OllamaProvider> {
        // Safe downcast helper if needed
        None
    }
}
