#[cfg_attr(not(test), allow(dead_code))]
#[must_use]
pub fn fuzzy_matches(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let haystack: Vec<char> = haystack.chars().flat_map(|ch| ch.to_lowercase()).collect();
    let needle: Vec<char> = needle.chars().flat_map(|ch| ch.to_lowercase()).collect();
    let mut hay_index = 0usize;

    for needle_ch in needle {
        while hay_index < haystack.len() && haystack[hay_index] != needle_ch {
            hay_index += 1;
        }
        if hay_index >= haystack.len() {
            return false;
        }
        hay_index += 1;
    }

    true
}

#[must_use]
pub fn fuzzy_score(haystack: &str, needle: &str) -> Option<u32> {
    if needle.is_empty() {
        return Some(0);
    }

    let haystack_chars: Vec<char> = haystack.chars().flat_map(|ch| ch.to_lowercase()).collect();
    let needle_chars: Vec<char> = needle.chars().flat_map(|ch| ch.to_lowercase()).collect();

    let mut hay_index = 0usize;
    let mut score = 0u32;
    let mut previous_match = None;

    for needle_ch in needle_chars {
        let mut matched = false;
        while hay_index < haystack_chars.len() {
            if haystack_chars[hay_index] == needle_ch {
                score += 10;
                if hay_index == 0 {
                    score += 20;
                }
                if previous_match == Some(hay_index.saturating_sub(1)) {
                    score += 15;
                }
                previous_match = Some(hay_index);
                hay_index += 1;
                matched = true;
                break;
            }
            hay_index += 1;
        }

        if !matched {
            return None;
        }
    }

    Some(score)
}

#[must_use]
pub fn best_fuzzy_score(id: &str, title: &str, query: &str) -> Option<u32> {
    match (fuzzy_score(id, query), fuzzy_score(title, query)) {
        (None, None) => None,
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (Some(left), Some(right)) => Some(left.max(right)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matches_subsequence() {
        assert!(fuzzy_matches("Git: Refresh Status", "git ref"));
        assert!(!fuzzy_matches("Git: Refresh Status", "github"));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(fuzzy_matches("anything", ""));
        assert_eq!(fuzzy_score("anything", ""), Some(0));
    }

    #[test]
    fn git_ref_scores_git_refresh_title() {
        assert!(fuzzy_matches("Git: Refresh Status", "git ref"));
        assert!(best_fuzzy_score("git.refresh", "Git: Refresh Status", "git ref").is_some());
    }

    #[test]
    fn fuzzy_score_prefers_prefix_matches() {
        let prefix = fuzzy_score("git refresh", "git").expect("score");
        let suffix = fuzzy_score("refresh git", "git").expect("score");
        assert!(prefix > suffix);
    }
}
