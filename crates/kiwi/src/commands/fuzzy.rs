//! Fuzzy subsequence matcher for the command palette (SPEC-013 / #28).
//!
//! Scoring favors consecutive matches, word boundaries, and early positions.
//! Designed for interactive filtering at palette typing speed.

const SCORE_MATCH: i32 = 16;
const SCORE_GAP: i32 = 2;
const BONUS_WORD_BOUNDARY: i32 = 12;
const BONUS_CONSECUTIVE: i32 = 10;
const BONUS_START: i32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyMatch {
    pub score: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[must_use]
pub fn fuzzy_matches(haystack: &str, needle: &str) -> bool {
    fuzzy_score(haystack, needle).is_some()
}

#[must_use]
pub fn fuzzy_score(haystack: &str, needle: &str) -> Option<FuzzyMatch> {
    let needle = normalize_needle(needle);
    if needle.is_empty() {
        return Some(FuzzyMatch { score: 0 });
    }

    let haystack = normalize_haystack(haystack);
    let mut hay_index = 0usize;
    let mut score = 0i32;
    let mut previous_match = None;

    for needle_ch in needle.chars() {
        let mut matched = false;
        while hay_index < haystack.len() {
            if haystack[hay_index] == needle_ch {
                score += SCORE_MATCH;
                if hay_index == 0 {
                    score += BONUS_START;
                }
                if is_word_boundary(&haystack, hay_index) {
                    score += BONUS_WORD_BOUNDARY;
                }
                if previous_match.is_some_and(|prev| prev + 1 == hay_index) {
                    score += BONUS_CONSECUTIVE;
                } else if let Some(prev) = previous_match {
                    let gap = hay_index.saturating_sub(prev + 1);
                    score -= SCORE_GAP * gap as i32;
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

    Some(FuzzyMatch { score })
}

#[must_use]
pub fn best_fuzzy_score(id: &str, title: &str, query: &str) -> Option<FuzzyMatch> {
    match (fuzzy_score(id, query), fuzzy_score(title, query)) {
        (None, None) => None,
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (Some(left), Some(right)) => Some(if left.score >= right.score {
            left
        } else {
            right
        }),
    }
}

#[must_use]
pub fn filter_ranked(
    count: usize,
    query: &str,
    mut score_at: impl FnMut(usize) -> Option<FuzzyMatch>,
) -> Vec<(usize, i32)> {
    let query = query.trim();
    if query.is_empty() {
        return (0..count).map(|index| (index, 0)).collect();
    }

    let mut ranked = Vec::new();
    for index in 0..count {
        if let Some(result) = score_at(index) {
            ranked.push((index, result.score));
        }
    }

    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    ranked
}

fn normalize_needle(needle: &str) -> String {
    needle
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn normalize_haystack(haystack: &str) -> Vec<char> {
    haystack.chars().flat_map(|ch| ch.to_lowercase()).collect()
}

fn is_word_boundary(haystack: &[char], index: usize) -> bool {
    if index == 0 {
        return true;
    }

    let current = haystack[index];
    let previous = haystack[index - 1];

    if !previous.is_alphanumeric() {
        return true;
    }

    if previous.is_ascii_lowercase() && current.is_ascii_uppercase() {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn fuzzy_matches_subsequence() {
        assert!(fuzzy_matches("Git: Refresh Status", "git ref"));
        assert!(!fuzzy_matches("Git: Refresh Status", "github"));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(fuzzy_matches("anything", ""));
        assert_eq!(fuzzy_score("anything", "").expect("score").score, 0);
    }

    #[test]
    fn git_ref_scores_git_refresh_title() {
        assert!(fuzzy_matches("Git: Refresh Status", "git ref"));
        assert!(best_fuzzy_score("git.refresh", "Git: Refresh Status", "git ref").is_some());
    }

    #[test]
    fn fuzzy_score_prefers_prefix_matches() {
        let prefix = fuzzy_score("git refresh", "git").expect("score").score;
        let suffix = fuzzy_score("refresh git", "git").expect("score").score;
        assert!(prefix > suffix);
    }

    #[test]
    fn git_ref_ranks_refresh_above_left_git() {
        let refresh = best_fuzzy_score("git.refresh", "Git: Refresh Status", "git ref")
            .expect("refresh")
            .score;
        let left_git =
            best_fuzzy_score("left.git", "Left Tab: Git", "git ref").map(|result| result.score);
        assert!(left_git.is_none() || refresh > left_git.expect("left git score"));
    }

    #[test]
    fn word_boundary_bonus_prefers_token_starts() {
        let boundary = fuzzy_score(" refresh", "ref").expect("score").score;
        let interior = fuzzy_score("xrefresh", "ref").expect("score").score;
        assert!(boundary > interior);
    }

    #[test]
    fn filter_ranked_sorts_by_score_descending() {
        let ranked = filter_ranked(3, "git", |index| match index {
            0 => fuzzy_score("quit", "git"),
            1 => fuzzy_score("git.refresh", "git"),
            2 => fuzzy_score("left.git", "git"),
            _ => None,
        });
        assert_eq!(ranked.first().map(|(index, _)| *index), Some(1));
    }

    #[test]
    fn filter_updates_within_spec_budget_for_100_commands() {
        let commands: Vec<String> = (0..100)
            .map(|index| format!("Command: Test Item {index:03}"))
            .collect();

        let start = Instant::now();
        for _ in 0..50 {
            let _ = filter_ranked(commands.len(), "test it", |index| {
                fuzzy_score(&commands[index], "test it")
            });
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(250),
            "filtering 100 commands should stay well under 5ms per update, took {elapsed:?}"
        );
    }
}
