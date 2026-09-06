use crate::protocol::CandidateSuggestion;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry {
    suggestions: Vec<CandidateSuggestion>,
    created_at: Instant,
}

pub struct SuggestionCache {
    entries: Mutex<HashMap<(String, PathBuf), CacheEntry>>,
    ttl: Duration,
    max_items: usize,
}

impl SuggestionCache {
    pub fn new(ttl_secs: u64, max_items: usize) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
            max_items,
        }
    }

    pub fn get(&self, input: &str, cwd: &PathBuf) -> Option<Vec<CandidateSuggestion>> {
        let key = (input.trim().to_lowercase(), cwd.clone());
        let mut map = self.entries.lock().ok()?;

        if let Some(entry) = map.get(&key) {
            if entry.created_at.elapsed() < self.ttl {
                return Some(entry.suggestions.clone());
            } else {
                map.remove(&key);
            }
        }
        None
    }

    pub fn insert(&self, input: &str, cwd: &PathBuf, suggestions: Vec<CandidateSuggestion>) {
        let key = (input.trim().to_lowercase(), cwd.clone());
        let mut map = match self.entries.lock() {
            Ok(m) => m,
            Err(_) => return,
        };

        if map.len() >= self.max_items {
            // Remove oldest or random entry
            if let Some(oldest_key) = map.keys().next().cloned() {
                map.remove(&oldest_key);
            }
        }

        map.insert(
            key,
            CacheEntry {
                suggestions,
                created_at: Instant::now(),
            },
        );
    }

    pub fn clear(&self) {
        if let Ok(mut map) = self.entries.lock() {
            map.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|m| m.len()).unwrap_or(0)
    }
}
