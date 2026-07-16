use voxflow_config::OutputMode;

/// Cheap, local, rule-based normalization applied before the AI rewrite pass
/// (and as the only cleanup when the rewrite pass is disabled or fails).
pub fn apply_rules(text: &str, mode: OutputMode) -> String {
    match mode {
        OutputMode::PlainText | OutputMode::TerminalSafe => text
            .replace(['\u{201c}', '\u{201d}'], "\"")
            .replace(['\u{2018}', '\u{2019}'], "'"),
        OutputMode::Casual => text.trim().to_string(),
        OutputMode::CodePreserve => text.to_string(),
        _ => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return String::new();
            }
            let mut chars = trimmed.chars();
            let first = chars.next().unwrap().to_uppercase().collect::<String>();
            format!("{first}{}", chars.as_str())
        }
    }
}

/// Builds the system prompt sent to the AI rewrite call, layering per-app
/// output-mode guidance on top of the user's base rewrite prompt.
pub fn system_prompt_for_mode(mode: OutputMode, base_prompt: &str) -> String {
    let suffix = match mode {
        OutputMode::CodePreserve => {
            "Preserve code terms, variable names, and technical vocabulary literally. Do not over-capitalize."
        }
        OutputMode::TerminalSafe => {
            "Output plain text only. No smart quotes, no markdown, no formatting. Never add a trailing command or instruction to execute anything."
        }
        OutputMode::Casual => "Keep a casual, short tone suitable for chat.",
        OutputMode::Email => "Use a professional tone with clean punctuation.",
        OutputMode::Markdown => "Format using markdown: headings, bullets, or todo items where natural.",
        OutputMode::PlainText | OutputMode::Balanced => "",
    };

    if suffix.is_empty() {
        base_prompt.to_string()
    } else {
        format!("{base_prompt}\n\n{suffix}")
    }
}

pub fn apply_snippets(text: &str, snippets: &[voxflow_config::Snippet]) -> String {
    let mut result = text.to_string();
    for snippet in snippets {
        if result.contains(&snippet.trigger) {
            result = result.replace(&snippet.trigger, &snippet.expansion);
        }
    }
    result
}

pub fn apply_dictionary(text: &str, dictionary: &[voxflow_config::DictionaryEntry]) -> String {
    let mut result = text.to_string();
    for entry in dictionary {
        if let Some(replacement) = &entry.replacement {
            result = result.replace(&entry.term, replacement);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_safe_quotes() {
        let out = apply_rules("\u{201c}hello\u{201d}", OutputMode::TerminalSafe);
        assert!(out.contains('"'));
    }

    #[test]
    fn code_preserve_is_untouched() {
        let out = apply_rules("let x = foo();", OutputMode::CodePreserve);
        assert_eq!(out, "let x = foo();");
    }
}
