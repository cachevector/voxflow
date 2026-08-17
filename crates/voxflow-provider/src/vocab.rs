use voxflow_config::DictionaryEntry;

pub use voxflow_config::{
    DEFAULT_REWRITE_PROMPT as DEFAULT_CLEANUP_PROMPT,
    LEGACY_REWRITE_PROMPT as LEGACY_CLEANUP_PROMPT,
};

pub fn default_cleanup_prompt() -> &'static str {
    DEFAULT_CLEANUP_PROMPT
}

/// Spellings Whisper should prefer. Fed as `initial_prompt` so the decoder
/// is biased toward these tokens instead of common-English lookalikes.
const BUILTIN_TERMS: &[&str] = &[
    "handoff",
    "LeetCode",
    "Zellij",
    "Ghostty",
    "iTerm2",
    "Svelte",
    "Remotion",
    "Flutter",
    "Xcode",
    "TypeScript",
    "JavaScript",
    "GitHub",
    "GitLab",
    "VS Code",
    "Cursor",
    "Groq",
    "Whisper",
    "VoxFlow",
    "Tailwind",
    "React",
    "Next.js",
    "Tauri",
    "Rust",
    "SwiftUI",
    "TestFlight",
    "App Store",
    "pnpm",
    "Homebrew",
    "Neovim",
    "tmux",
    "Docker",
    "Postgres",
    "SQLite",
    "Kubernetes",
    "GraphQL",
    "WebSocket",
    "OAuth",
    "JSON",
    "YAML",
    "TOML",
];

/// High-confidence post-STT phrase fixes. Only mappings that are almost never
/// valid English as written. Ambiguous splits like "hand of" stay out — those
/// are handled by the Whisper prompt and the rewrite pass.
const BUILTIN_CORRECTIONS: &[(&str, &str)] = &[
    ("leet code", "LeetCode"),
    ("lead code", "LeetCode"),
    ("zelich", "Zellij"),
    ("zellich", "Zellij"),
    ("zelij", "Zellij"),
    ("swelt", "Svelte"),
    ("iterm2", "iTerm2"),
    ("i term 2", "iTerm2"),
    ("vs code", "VS Code"),
    ("next js", "Next.js"),
    ("nextjs", "Next.js"),
    ("node js", "Node.js"),
    ("nodejs", "Node.js"),
    ("x code", "Xcode"),
    ("test flight", "TestFlight"),
    ("type script", "TypeScript"),
    ("java script", "JavaScript"),
    ("git hub", "GitHub"),
    ("git lab", "GitLab"),
];

/// Decoder prefix for whisper.cpp. Kept short — the prompt window is ~224 tokens.
pub fn whisper_initial_prompt(dictionary: &[DictionaryEntry]) -> String {
    let mut terms: Vec<String> = BUILTIN_TERMS.iter().map(|s| (*s).to_string()).collect();
    for entry in dictionary {
        let preferred = entry.replacement.as_deref().unwrap_or(&entry.term);
        let preferred = preferred.trim();
        if preferred.is_empty() {
            continue;
        }
        if !terms.iter().any(|t| t.eq_ignore_ascii_case(preferred)) {
            terms.push(preferred.to_string());
        }
    }
    format!(
        "Software engineering dictation. Vocabulary: {}.",
        terms.join(", ")
    )
}

/// Extra system-prompt block so the rewrite model sees both preferred spellings
/// and the Whisper mistakes it should undo.
pub fn rewrite_vocab_suffix(dictionary: &[DictionaryEntry]) -> String {
    let mut lines = vec![
        "You are cleaning speech from a software engineer. Whisper often splits or misspells technical terms. Repair those when the intended term is clear from context.".to_string(),
        "Examples: \"hand of\"/\"hand off\" → handoff (transferring work, not a literal hand); \"lead code\"/\"leet code\" → LeetCode; \"zelich\" → Zellij; \"ghosty\" → Ghostty; \"swelt\" → Svelte; \"emotion video\" (when about code) → Remotion.".to_string(),
        format!("Prefer these spellings when they fit: {}.", BUILTIN_TERMS.join(", ")),
    ];

    let user_terms: Vec<&str> = dictionary
        .iter()
        .map(|e| e.replacement.as_deref().unwrap_or(&e.term))
        .filter(|t| !t.trim().is_empty())
        .collect();
    if !user_terms.is_empty() {
        lines.push(format!(
            "Also prefer these user vocabulary terms: {}.",
            user_terms.join(", ")
        ));
    }

    lines.join(" ")
}

/// Applies built-in high-confidence phrase fixes, then the user's dictionary
/// replacements. Terms without a replacement are vocab-only (Whisper/rewrite).
pub fn apply_vocabulary(text: &str, dictionary: &[DictionaryEntry]) -> String {
    let mut pairs: Vec<(String, String)> = BUILTIN_CORRECTIONS
        .iter()
        .map(|(from, to)| ((*from).to_string(), (*to).to_string()))
        .collect();

    for entry in dictionary {
        if let Some(replacement) = &entry.replacement {
            let from = entry.term.trim();
            let to = replacement.trim();
            if !from.is_empty() && !to.is_empty() {
                pairs.push((from.to_string(), to.to_string()));
            }
        }
    }

    apply_phrase_replacements(text, &pairs)
}

/// Case-insensitive, word-boundary replacements. Longer needles run first so
/// "leet code" wins over a hypothetical "leet".
pub fn apply_phrase_replacements(text: &str, pairs: &[(String, String)]) -> String {
    let mut ordered: Vec<&(String, String)> = pairs.iter().collect();
    ordered.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

    let mut result = text.to_string();
    for (from, to) in ordered {
        result = replace_whole_phrase(&result, from, to);
    }
    result
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn replace_whole_phrase(text: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() || text.is_empty() {
        return text.to_string();
    }

    let hay_lower: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let needle_lower: Vec<char> = needle.to_ascii_lowercase().chars().collect();
    let original: Vec<char> = text.chars().collect();
    if needle_lower.len() > hay_lower.len() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < hay_lower.len() {
        let remaining = hay_lower.len() - i;
        if remaining >= needle_lower.len()
            && hay_lower[i..i + needle_lower.len()] == needle_lower[..]
        {
            let start_ok = i == 0 || !is_word_char(original[i.saturating_sub(1)]);
            let end = i + needle_lower.len();
            let end_ok = end == original.len() || !is_word_char(original[end]);
            if start_ok && end_ok {
                out.push_str(replacement);
                i = end;
                continue;
            }
        }
        out.push(original[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whisper_prompt_includes_builtin_and_user_terms() {
        let dict = vec![DictionaryEntry {
            term: "mass syntax".into(),
            replacement: Some("Masked Syntax".into()),
        }];
        let prompt = whisper_initial_prompt(&dict);
        assert!(prompt.contains("handoff"));
        assert!(prompt.contains("LeetCode"));
        assert!(prompt.contains("Masked Syntax"));
    }

    #[test]
    fn builtin_fixes_leetcode_split() {
        let out = apply_vocabulary("practice these from lead code only", &[]);
        assert_eq!(out, "practice these from LeetCode only");

        let out = apply_vocabulary("I use leet code daily", &[]);
        assert_eq!(out, "I use LeetCode daily");
    }

    #[test]
    fn does_not_touch_hand_of() {
        // Ambiguous English — must not become "handoff" via hard replace.
        let out = apply_vocabulary("the hand of the king", &[]);
        assert_eq!(out, "the hand of the king");
    }

    #[test]
    fn user_dictionary_replaces_phrase() {
        let dict = vec![DictionaryEntry {
            term: "mass syntax".into(),
            replacement: Some("Masked Syntax".into()),
        }];
        let out = apply_vocabulary("repo of mass syntax is private", &dict);
        assert_eq!(out, "repo of Masked Syntax is private");
    }

    #[test]
    fn dictionary_term_without_replacement_is_ignored_here() {
        let dict = vec![DictionaryEntry {
            term: "handoff".into(),
            replacement: None,
        }];
        let out = apply_vocabulary("do the handoff now", &dict);
        assert_eq!(out, "do the handoff now");
    }

    #[test]
    fn replacement_is_case_insensitive_and_bounded() {
        let dict = vec![DictionaryEntry {
            term: "swelt".into(),
            replacement: Some("Svelte".into()),
        }];
        let out = apply_vocabulary("SWELT and swelter", &dict);
        assert_eq!(out, "Svelte and swelter");
    }

    #[test]
    fn rewrite_suffix_mentions_user_vocab() {
        let dict = vec![DictionaryEntry {
            term: "gentle loop".into(),
            replacement: Some("Gentle Loop".into()),
        }];
        let suffix = rewrite_vocab_suffix(&dict);
        assert!(suffix.contains("Gentle Loop"));
        assert!(suffix.contains("handoff"));
    }
}
