use crate::config::Config;
use crate::protocol::CandidateSuggestion;
use crate::safety::check_safety;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningRecord {
    pub input: String,
    pub command: String,
    pub timestamp: i64,
}

pub struct LearningEngine {
    file_path: PathBuf,
}

impl LearningEngine {
    pub fn new() -> Self {
        let dir = Config::data_dir();
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("accepted_history.jsonl");
        Self { file_path }
    }

    pub fn record_accepted(&self, input: &str, command: &str) -> std::io::Result<()> {
        let rec = LearningRecord {
            input: input.trim().to_lowercase(),
            command: command.trim().to_string(),
            timestamp: Utc::now().timestamp(),
        };

        let json = serde_json::to_string(&rec)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", json)?;
        Ok(())
    }

    pub fn lookup(&self, input: &str) -> Vec<CandidateSuggestion> {
        let target = input.trim().to_lowercase();
        if target.is_empty() || !self.file_path.exists() {
            return Vec::new();
        }

        let file = match fs::File::open(&self.file_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let mut counts: HashMap<String, usize> = HashMap::new();

        for line in reader.lines().flatten() {
            if let Ok(rec) = serde_json::from_str::<LearningRecord>(&line) {
                if rec.input == target {
                    *counts.entry(rec.command).or_insert(0) += 1;
                }
            }
        }

        let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        sorted
            .into_iter()
            .take(2)
            .map(|(cmd, count)| {
                let safety = check_safety(&cmd);
                CandidateSuggestion {
                    command: safety.normalized_command,
                    description: format!("Previously accepted command (used {}x)", count),
                    confidence: 0.99,
                    risk: safety.risk,
                    warning: safety.warning,
                    source: "history".to_string(),
                    category: Some("History".to_string()),
                }
            })
            .collect()
    }
}
