use crate::context::SystemContext;
use crate::protocol::CandidateSuggestion;
use crate::provider::AIProvider;
use crate::safety::check_safety;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaProvider {
    endpoint: String,
    model_override: String,
    client: Client,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Option<Vec<OllamaModelItem>>,
}

#[derive(Deserialize)]
struct OllamaModelItem {
    name: String,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
    options: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Deserialize, Serialize)]
struct ParsedAiSuggestion {
    command: String,
    description: Option<String>,
    confidence: Option<f32>,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String, timeout_ms: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model_override: model,
            client,
        }
    }

    pub async fn select_active_model(&self) -> Result<String, String> {
        if !self.model_override.is_empty() && self.model_override != "auto" {
            return Ok(self.model_override.clone());
        }

        let models = self.list_models().await?;
        if models.is_empty() {
            return Err("No Ollama models installed. Pull a model (e.g. `ollama pull qwen2.5-coder:1.5b`) to enable AI suggestions.".to_string());
        }

        // Pick preferred coder/command models
        let priority_keywords = ["qwen2.5-coder", "coder", "qwen", "deepseek", "codellama", "mistral", "llama"];
        for keyword in priority_keywords {
            if let Some(m) = models.iter().find(|m| m.to_lowercase().contains(keyword)) {
                return Ok(m.clone());
            }
        }

        Ok(models[0].clone())
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Could not connect to Ollama ({}): {}", url, e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned status {}", resp.status()));
        }

        let data: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama models response: {}", e))?;

        let list = data
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.name)
            .collect();

        Ok(list)
    }

    async fn suggest(&self, input: &str, ctx: &SystemContext) -> Result<Vec<CandidateSuggestion>, String> {
        let model = self.select_active_model().await?;

        let system_prompt = format!(
            "You are a native Linux terminal command generation engine for CachyOS (an Arch Linux derivative). \
            The user shell is fish. Desktop is Hyprland on Wayland. \
            Package manager is pacman for official repos and paru for AUR. \
            CRITICAL RULES: \
            1. NEVER suggest Ubuntu/Debian commands (NEVER use 'apt', 'apt-get', etc.). \
            2. Output MUST be ONLY valid JSON matching this exact structure: \
            {{\"suggestions\": [{{\"command\": \"...\", \"description\": \"...\", \"confidence\": 0.95}}]}} \
            3. Do not include markdown code block formatting (no ```json or ```). Return raw JSON only. \
            4. Provide 1 to 3 distinct relevant command candidates.",
        );

        let files_summary = if ctx.files.is_empty() {
            "none".to_string()
        } else {
            ctx.files.iter().take(15).cloned().collect::<Vec<_>>().join(", ")
        };

        let user_prompt = format!(
            "Intent: {}\n\
            Context:\n\
            - OS: {}\n\
            - Shell: {}\n\
            - CWD: {}\n\
            - Files in CWD: {}\n\
            - In Git repo: {}\n\
            Generate the best matching Linux command(s):",
            input,
            ctx.os_id,
            ctx.shell,
            ctx.cwd.display(),
            files_summary,
            ctx.is_git_repo,
        );

        let req_body = OllamaGenerateRequest {
            model: model.clone(),
            prompt: user_prompt,
            system: system_prompt,
            stream: false,
            options: Some(serde_json::json!({
                "temperature": 0.1,
                "top_p": 0.8,
                "stop": ["\n\n"]
            })),
        };

        let url = format!("{}/api/generate", self.endpoint);
        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("Ollama generation failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama HTTP status: {}", resp.status()));
        }

        let gen_resp: OllamaGenerateResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to decode Ollama response JSON: {}", e))?;

        let clean_json = extract_json(&gen_resp.response);

        // Try parsing {"suggestions": [...]}
        #[derive(Deserialize)]
        struct Envelope {
            suggestions: Option<Vec<ParsedAiSuggestion>>,
        }

        let mut parsed_list = Vec::new();
        if let Ok(env) = serde_json::from_str::<Envelope>(&clean_json) {
            if let Some(list) = env.suggestions {
                parsed_list = list;
            }
        }

        // If envelope didn't work, try parsing direct array or single object
        if parsed_list.is_empty() {
            if let Ok(list) = serde_json::from_str::<Vec<ParsedAiSuggestion>>(&clean_json) {
                parsed_list = list;
            } else if let Ok(single) = serde_json::from_str::<ParsedAiSuggestion>(&clean_json) {
                parsed_list = vec![single];
            }
        }

        // If JSON parse failed, try extracting raw single command line
        if parsed_list.is_empty() {
            let line = gen_resp.response.lines().next().unwrap_or("").trim().trim_matches('`');
            if !line.is_empty() && !line.starts_with('{') {
                parsed_list.push(ParsedAiSuggestion {
                    command: line.to_string(),
                    description: Some("AI generated command".to_string()),
                    confidence: Some(0.85),
                });
            }
        }

        let mut results = Vec::new();
        for item in parsed_list {
            let raw_cmd = item.command.trim();
            if raw_cmd.is_empty() {
                continue;
            }

            let safety = check_safety(raw_cmd);
            let mut conf = item.confidence.unwrap_or(0.90) - safety.confidence_penalty;
            if conf < 0.1 {
                conf = 0.1;
            }

            results.push(CandidateSuggestion {
                command: safety.normalized_command,
                description: item.description.unwrap_or_else(|| "Generated Linux command".to_string()),
                confidence: conf,
                risk: safety.risk,
                warning: safety.warning,
                source: "ai".to_string(),
                category: Some("AI".to_string()),
            });
        }

        Ok(results)
    }

    async fn explain(&self, command: &str) -> Result<String, String> {
        let model = self.select_active_model().await?;

        let prompt = format!(
            "Explain this Linux/CachyOS command concisely in 1-2 plain sentences. Mention any flags and safety impact.\nCommand: {}",
            command
        );

        let req_body = OllamaGenerateRequest {
            model,
            prompt,
            system: "You are a concise Linux terminal expert. Provide short, accurate 1-2 sentence explanations with no pleasantries.".to_string(),
            stream: false,
            options: Some(serde_json::json!({
                "temperature": 0.2
            })),
        };

        let url = format!("{}/api/generate", self.endpoint);
        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("Ollama explain request failed: {}", e))?;

        let gen_resp: OllamaGenerateResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to read explain response: {}", e))?;

        Ok(gen_resp.response.trim().to_string())
    }
}

fn extract_json(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start + 7..].find("```") {
            return trimmed[start + 7..start + 7 + end].trim().to_string();
        }
    } else if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed[start + 3..].find("```") {
            return trimmed[start + 3..start + 3 + end].trim().to_string();
        }
    } else if let (Some(s), Some(e)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if s <= e {
            return trimmed[s..=e].to_string();
        }
    }
    trimmed.to_string()
}
