// Fast Damerau-Levenshtein fuzzy distance & similarity scoring
// and natural language intent preprocessing

pub fn damerau_levenshtein(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut d = vec![vec![0usize; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        d[i][0] = i;
    }
    for j in 0..=len2 {
        d[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };

            d[i][j] = (d[i - 1][j] + 1) // deletion
                .min(d[i][j - 1] + 1) // insertion
                .min(d[i - 1][j - 1] + cost); // substitution

            // Transposition (e.g. 'teh' -> 'the')
            if i > 1 && j > 1 && s1_chars[i - 1] == s2_chars[j - 2] && s1_chars[i - 2] == s2_chars[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }

    d[len1][len2]
}

pub fn similarity(s1: &str, s2: &str) -> f32 {
    let max_len = s1.chars().count().max(s2.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let dist = damerau_levenshtein(s1, s2);
    1.0 - (dist as f32 / max_len as f32)
}

pub fn matches_fuzzy(input: &str, target: &str, threshold: f32) -> bool {
    if input == target || target.starts_with(input) {
        return true;
    }
    similarity(input, target) >= threshold
}

pub fn fuzzy_find_best<'a>(input: &str, candidates: &[&'a str], min_score: f32) -> Option<(&'a str, f32)> {
    let mut best_match = None;
    let mut best_score = min_score;

    for &candidate in candidates {
        let score = similarity(input, candidate);
        if score > best_score {
            best_score = score;
            best_match = Some(candidate);
        }
    }

    best_match.map(|m| (m, best_score))
}

// -----------------------------------------------------------------------------
// Natural Language Preprocessing & Conversational Preamble Stripping
// -----------------------------------------------------------------------------

const CONVERSATIONAL_PREFIXES: &[&str] = &[
    "how to",
    "how do i",
    "how can i",
    "how should i",
    "how do we",
    "can you please",
    "can you",
    "could you please",
    "could you",
    "would you please",
    "would you",
    "i want to",
    "i need to",
    "i want",
    "i need",
    "help me",
    "tell me",
    "show me",
    "give me",
    "let me",
    "check if",
    "check the",
    "check out",
    "what is",
    "what are",
    "whats",
    "what's",
    "is there a way to",
    "how about",
    "can i",
    "please",
    "pls",
    "plz",
];

const FILLER_WORDS: &[&str] = &[
    "the", "a", "an", "my", "our", "your", "this", "some", "any", "please", "pls", "plz", "just",
];

/// Cleans and normalizes conversational sentences by stripping preambles,
/// punctuation, and noise tokens while preserving meaningful commands and arguments.
pub fn clean_intent(input: &str) -> (String, Vec<String>) {
    // 1. Lowercase and strip punctuation (?, !, ,, ;) but keep dashes, underscores, dots, slashes
    let mut cleaned = input.to_lowercase();
    cleaned = cleaned
        .chars()
        .map(|c| if c == '?' || c == '!' || c == ',' || c == ';' || c == ':' || c == '"' || c == '\'' { ' ' } else { c })
        .collect();

    let mut trimmed = cleaned.trim().to_string();

    // 2. Strip multi-word prefixes
    for prefix in CONVERSATIONAL_PREFIXES {
        if trimmed.starts_with(prefix) {
            let rem = trimmed[prefix.len()..].trim();
            if !rem.is_empty() {
                trimmed = rem.to_string();
                break;
            }
        }
    }

    // 3. Tokenize
    let raw_tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mut filtered_tokens: Vec<String> = Vec::new();

    for (_i, &t) in raw_tokens.iter().enumerate() {
        // Only strip pure filler words if there are other tokens present
        if raw_tokens.len() > 1 && FILLER_WORDS.contains(&t) {
            // Keep if it looks like part of a filename or flag (e.g. -a)
            if t.starts_with('-') {
                filtered_tokens.push(t.to_string());
            }
            continue;
        }
        filtered_tokens.push(t.to_string());
    }

    if filtered_tokens.is_empty() && !raw_tokens.is_empty() {
        filtered_tokens = raw_tokens.into_iter().map(String::from).collect();
    }

    let normalized_str = filtered_tokens.join(" ");
    (normalized_str, filtered_tokens)
}

// -----------------------------------------------------------------------------
// Action Verbs Classification
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionVerb {
    Install,
    Remove,
    Update,
    Restart,
    Query,
    Kill,
    Extract,
    PowerReboot,
    PowerShutdown,
    PowerSleep,
    PowerLock,
    PowerLogout,
    Clean,
    Fix,
}

pub fn classify_action_token(token: &str) -> Option<ActionVerb> {
    let t = token.to_lowercase();
    match t.as_str() {
        // Install
        "install" | "instal" | "isntall" | "instll" | "intall" | "get" | "download" | "dwonload"
        | "fetch" | "setup" | "grab" | "add" => Some(ActionVerb::Install),

        // Remove
        "remove" | "rm" | "delete" | "delet" | "del" | "uninstall" | "unistall" | "purge"
        | "erase" => Some(ActionVerb::Remove),

        // Update
        "update" | "updat" | "upgrade" | "upgrd" | "refresh" | "sync" | "patch" => {
            Some(ActionVerb::Update)
        }

        // Restart
        "restart" | "restrt" | "reload" | "bounce" | "reset" => Some(ActionVerb::Restart),

        // Query
        "check" | "chekc" | "show" | "status" | "stat" | "staus" | "monitor" | "view" | "info"
        | "find" | "search" | "lookup" | "list" | "scan" | "inspect" | "see" | "what" => {
            Some(ActionVerb::Query)
        }

        // Kill / Stop
        "kill" | "stop" | "terminate" | "end" | "close" => Some(ActionVerb::Kill),

        // Extract
        "extract" | "extarct" | "unzip" | "untar" | "decompress" | "unpack" => {
            Some(ActionVerb::Extract)
        }

        // Power
        "reboot" | "rebot" => Some(ActionVerb::PowerReboot),
        "shutdown" | "shut" | "poweroff" | "pwerof" => Some(ActionVerb::PowerShutdown),
        "sleep" | "suspend" | "hibernate" => Some(ActionVerb::PowerSleep),
        "lock" => Some(ActionVerb::PowerLock),
        "logout" | "exit" => Some(ActionVerb::PowerLogout),

        // Clean
        "clean" | "cleen" | "clear" | "prune" => Some(ActionVerb::Clean),

        // Fix
        "fix" | "repair" | "mend" => Some(ActionVerb::Fix),

        _ => {
            // Fuzzy similarity fallback for typos
            if similarity(&t, "install") >= 0.75 {
                Some(ActionVerb::Install)
            } else if similarity(&t, "uninstall") >= 0.75 || similarity(&t, "remove") >= 0.75 {
                Some(ActionVerb::Remove)
            } else if similarity(&t, "restart") >= 0.75 {
                Some(ActionVerb::Restart)
            } else if similarity(&t, "upgrade") >= 0.75 || similarity(&t, "update") >= 0.75 {
                Some(ActionVerb::Update)
            } else if similarity(&t, "status") >= 0.75 || similarity(&t, "check") >= 0.75 {
                Some(ActionVerb::Query)
            } else if similarity(&t, "poweroff") >= 0.75 || similarity(&t, "shutdown") >= 0.75 {
                Some(ActionVerb::PowerShutdown)
            } else if similarity(&t, "reboot") >= 0.75 {
                Some(ActionVerb::PowerReboot)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_typos() {
        assert_eq!(damerau_levenshtein("instal", "install"), 1);
        assert_eq!(damerau_levenshtein("chekc", "check"), 1); // transposition
        assert_eq!(damerau_levenshtein("restrt", "restart"), 1);
        assert_eq!(damerau_levenshtein("chorme", "chrome"), 1);
    }

    #[test]
    fn test_similarity() {
        assert!(similarity("instal", "install") > 0.85);
        assert!(similarity("chekc", "check") > 0.79);
        assert!(similarity("restrt", "restart") > 0.85);
    }

    #[test]
    fn test_clean_intent() {
        let (cleaned, tokens) = clean_intent("how to install google chrome?");
        assert_eq!(cleaned, "install google chrome");
        assert_eq!(tokens, vec!["install", "google", "chrome"]);

        let (cleaned2, tokens2) = clean_intent("can you please check the gpu!");
        assert_eq!(cleaned2, "check gpu");
        assert_eq!(tokens2, vec!["check", "gpu"]);

        let (cleaned3, tokens3) = clean_intent("could you please restart my audio");
        assert_eq!(cleaned3, "restart audio");
        assert_eq!(tokens3, vec!["restart", "audio"]);
    }

    #[test]
    fn test_action_classification() {
        assert_eq!(classify_action_token("isntall"), Some(ActionVerb::Install));
        assert_eq!(classify_action_token("restrt"), Some(ActionVerb::Restart));
        assert_eq!(classify_action_token("chekc"), Some(ActionVerb::Query));
        assert_eq!(classify_action_token("del"), Some(ActionVerb::Remove));
        assert_eq!(classify_action_token("rebot"), Some(ActionVerb::PowerReboot));
    }
}
