use voxflow_config::OutputMode;

/// Default system prompt for the Groq cleanup pass (grammar + filler removal only).
pub fn default_cleanup_prompt() -> &'static str {
    "You clean up dictated speech. Remove filler words and false starts (um, uh, hmm, er). Fix grammar, punctuation, and capitalization. Do NOT change meaning, add content, or rephrase for style. Return ONLY the corrected text with no quotes or markdown."
}

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

/// Sentence-level cleanup applied to every dictation, even when the AI rewrite
/// pass is disabled or unavailable. This is what makes raw Whisper output read
/// like written text: it capitalizes the first letter, ends the utterance with
/// the right terminal punctuation (`?` when it looks like a question, `.`
/// otherwise), and appends a single trailing space so two consecutive
/// dictations don't collide into one run-on word.
///
/// Modes that carry literal payloads — code and terminal commands — are left
/// untouched, since a stray period or capital would corrupt them.
pub fn finalize_text(text: &str, mode: OutputMode) -> String {
    if matches!(mode, OutputMode::CodePreserve | OutputMode::TerminalSafe) {
        return text.to_string();
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut result = capitalize_first(&insert_missing_sentence_spaces(trimmed));

    // Only append terminal punctuation when the utterance doesn't already end
    // in a sentence terminator; an existing `,`/`:` etc. is left as the writer
    // dictated it (they may be mid-thought across dictations).
    let last_significant = result.chars().rev().find(|c| !c.is_whitespace());
    let already_terminated = matches!(last_significant, Some('.' | '!' | '?' | '…'));
    if !already_terminated {
        if is_question(trimmed) {
            result.push('?');
        } else {
            result.push('.');
        }
    }

    // Trailing space keeps the next dictation from butting up against this one.
    result.push(' ');
    result
}

/// Repairs sentences that were glued together with no space after the
/// terminator ("done.Next one" -> "done. Next one").
///
/// Deliberately narrow to avoid corrupting things that legitimately contain a
/// dot: a space is only inserted when the punctuation is preceded by a
/// *lowercase* letter and followed by an *uppercase* one. That leaves decimals
/// ("3.14"), file names ("main.rs"), and initialisms ("U.S.A") untouched.
fn insert_missing_sentence_spaces(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);

    for (i, &c) in chars.iter().enumerate() {
        out.push(c);
        if !matches!(c, '.' | '!' | '?' | ',' | ';' | ':') {
            continue;
        }
        let prev_is_lower = i
            .checked_sub(1)
            .and_then(|p| chars.get(p))
            .is_some_and(|p| p.is_lowercase());
        let next_is_upper = chars.get(i + 1).is_some_and(|n| n.is_uppercase());
        if prev_is_lower && next_is_upper {
            out.push(' ');
        }
    }
    out
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Heuristic question detection for the local (no-AI) path. Looks at the first
/// word: interrogatives (who/what/why…) and leading auxiliaries (is/do/can…)
/// almost always open a spoken question. Not perfect — "Will" as a name would
/// be a false positive — but a reasonable default, and the AI rewrite pass does
/// better when it's enabled.
fn is_question(text: &str) -> bool {
    let first_word: String = text
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphabetic() || *c == '\'')
        .flat_map(|c| c.to_lowercase())
        .collect();

    // Contractions are matched on their stem ("isn't" -> "isn", "don't" -> "don").
    const OPENERS: &[&str] = &[
        "who", "what", "when", "where", "why", "which", "whom", "whose", "how",
        "is", "are", "am", "was", "were", "do", "does", "did", "can", "could",
        "would", "should", "will", "shall", "may", "might", "have", "has", "had",
        "isn", "aren", "wasn", "weren", "don", "doesn", "didn", "can", "couldn",
        "wouldn", "shouldn", "won", "haven", "hasn", "hadn", "ain",
    ];

    // Match on the part before any apostrophe so contractions resolve to their
    // stem ("isn't" -> "isn", "don't" -> "don").
    let stem = first_word.split('\'').next().unwrap_or("");
    OPENERS.contains(&stem)
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

    #[test]
    fn finalize_capitalizes_and_terminates_statement() {
        assert_eq!(
            finalize_text("hello world", OutputMode::Balanced),
            "Hello world. "
        );
    }

    #[test]
    fn finalize_detects_questions() {
        assert_eq!(finalize_text("who am i", OutputMode::Balanced), "Who am i? ");
        assert_eq!(
            finalize_text("can you help me", OutputMode::Balanced),
            "Can you help me? "
        );
        assert_eq!(
            finalize_text("isn't it late", OutputMode::Balanced),
            "Isn't it late? "
        );
    }

    #[test]
    fn finalize_respects_existing_terminator() {
        assert_eq!(
            finalize_text("Stop right there!", OutputMode::Balanced),
            "Stop right there! "
        );
        assert_eq!(
            finalize_text("Already done.", OutputMode::Balanced),
            "Already done. "
        );
    }

    #[test]
    fn finalize_leaves_code_and_terminal_alone() {
        assert_eq!(
            finalize_text("ls -la", OutputMode::TerminalSafe),
            "ls -la"
        );
        assert_eq!(
            finalize_text("let x = 1", OutputMode::CodePreserve),
            "let x = 1"
        );
    }

    #[test]
    fn finalize_repairs_missing_space_after_terminator() {
        assert_eq!(
            finalize_text("this is done.Next one starts", OutputMode::Balanced),
            "This is done. Next one starts. "
        );
        assert_eq!(
            finalize_text("wait,Then go", OutputMode::Balanced),
            "Wait, Then go. "
        );
    }

    #[test]
    fn finalize_does_not_break_dotted_tokens() {
        // Decimals, filenames, and initialisms must survive untouched.
        assert_eq!(
            finalize_text("the value is 3.14 exactly", OutputMode::Balanced),
            "The value is 3.14 exactly. "
        );
        assert_eq!(
            finalize_text("open main.rs now", OutputMode::Balanced),
            "Open main.rs now. "
        );
        assert_eq!(
            finalize_text("I live in the U.S.A", OutputMode::Balanced),
            "I live in the U.S.A. "
        );
    }

    #[test]
    fn finalize_trailing_space_separates_dictations() {
        // Two consecutive dictations concatenate cleanly instead of running on.
        let a = finalize_text("first sentence", OutputMode::Balanced);
        let b = finalize_text("second sentence", OutputMode::Balanced);
        assert_eq!(format!("{a}{b}"), "First sentence. Second sentence. ");
    }
}
