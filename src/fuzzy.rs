// Fast Damerau-Levenshtein fuzzy distance & similarity scoring

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
}
