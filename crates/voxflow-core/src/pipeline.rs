use crate::events::{DictationState, StateEvent};
use crate::latency::LatencyTracker;
use anyhow::{Context, Result};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use voxflow_audio::AudioCapture;
use voxflow_config::{
    DictionaryEntry, OutputMode, ProviderConfig, Settings, VocabularySuggestionDismissal,
};
use voxflow_cost::{build_dashboard, cap_reached, MonthlyUsage, UsageRecord};
use voxflow_history::{new_entry, HistoryStore};
use voxflow_insert::{InsertResult, InsertionBridge};
use voxflow_provider::{
    apply_dictionary, apply_rules, apply_snippets, finalize_text, system_prompt_for_mode,
    whisper_initial_prompt,
};
use voxflow_vad::{VoiceActivityDetector, SAMPLE_RATE};
use voxflow_whisper::WhisperEngine;

pub struct DictationResult {
    pub text: String,
    pub state: DictationState,
    pub insert_result: Option<InsertResult>,
    pub latency_ms: u64,
}

/// An exact phrase replacement inferred from a small, intentional history edit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VocabularySuggestion {
    pub term: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryCorrectionResult {
    pub entry: voxflow_history::HistoryEntry,
    pub suggestion: Option<VocabularySuggestion>,
}

pub struct DictationPipeline {
    settings: Arc<RwLock<Settings>>,
    capture: Option<AudioCapture>,
    inserter: Arc<InsertionBridge>,
    whisper: Arc<WhisperEngine>,
    history: HistoryStore,
    monthly_usage: MonthlyUsage,
    latency: LatencyTracker,
    state: DictationState,
}

impl DictationPipeline {
    pub fn new(
        settings: Settings,
        inserter: Arc<InsertionBridge>,
        whisper: Arc<WhisperEngine>,
    ) -> Result<Self> {
        let history = HistoryStore::open().context("open history")?;
        let mut monthly_usage = MonthlyUsage::current();
        let now = chrono::Local::now();
        if let Ok(Some(json)) = history.load_monthly_usage(now.month(), now.year()) {
            if let Ok(loaded) = serde_json::from_str(&json) {
                monthly_usage = loaded;
            }
        }

        Ok(Self {
            settings: Arc::new(RwLock::new(settings)),
            capture: None,
            inserter,
            whisper,
            history,
            monthly_usage,
            latency: LatencyTracker::new(),
            state: DictationState::Idle,
        })
    }

    pub fn state(&self) -> DictationState {
        self.state
    }

    /// Current mic input level (0.0..1.0) for the live overlay waveform.
    /// Returns 0.0 when no capture is active.
    pub fn current_level(&self) -> f32 {
        self.capture
            .as_ref()
            .map(|c| c.current_level())
            .unwrap_or(0.0)
    }

    pub async fn prewarm(&mut self) -> Result<()> {
        let settings = self.settings.read().await.clone();
        let device = settings.microphone_device.clone();
        let capture = AudioCapture::prewarm(device.as_deref()).context("prewarm audio")?;
        self.capture = Some(capture);
        if settings.whisper.prewarm_on_launch {
            self.whisper
                .prewarm()
                .await
                .context("prewarm whisper model")?;
        }
        Ok(())
    }
    pub async fn start_listening(&mut self) -> StateEvent {
        self.latency.reset();
        self.latency.mark_hotkey_down();
        self.state = DictationState::Listening;

        if self.capture.is_none() {
            let device = self.settings.read().await.microphone_device.clone();
            match AudioCapture::open(device.as_deref()) {
                Ok(c) => self.capture = Some(c),
                Err(e) => {
                    self.state = DictationState::Error;
                    return StateEvent::new(self.state, Some(format!("Microphone error: {e}")));
                }
            }
        }

        self.latency.mark_recording_start();
        self.latency.mark_bar_shown();
        if let Some(capture) = self.capture.as_ref() {
            let _ = capture.drain_samples();
        }
        StateEvent::new(self.state, None)
    }

    pub async fn stop_and_process(
        &mut self,
        active_app_id: Option<String>,
    ) -> Result<(StateEvent, DictationResult)> {
        self.stop_and_process_inner(active_app_id).await
    }

    fn error_result(&mut self, message: impl Into<String>) -> (StateEvent, DictationResult) {
        self.state = DictationState::Error;
        (
            StateEvent::new(self.state, Some(message.into())),
            DictationResult {
                text: String::new(),
                state: self.state,
                insert_result: None,
                latency_ms: 0,
            },
        )
    }

    async fn resolve_client(config: &ProviderConfig) -> voxflow_provider::OpenAiCompatibleClient {
        let secret = match &config.api_key_ref {
            Some(key_ref) => voxflow_secrets::get_secret(key_ref).ok().flatten(),
            None => None,
        };
        voxflow_provider::build_client(config, secret)
    }

    async fn stop_and_process_inner(
        &mut self,
        active_app_id: Option<String>,
    ) -> Result<(StateEvent, DictationResult)> {
        self.latency.mark_hotkey_up();
        self.state = DictationState::Finalizing;

        let pcm_i16 = self
            .capture
            .as_ref()
            .map(|c| c.take_utterance_pcm_i16_after_flush())
            .unwrap_or_default();

        // Whisper returns "[BLANK_AUDIO]" for anything at room-noise level, which
        // is indistinguishable from a pipeline fault without the captured level.
        let capture_peak =
            pcm_i16.iter().fold(0i16, |m, &s| m.max(s.saturating_abs())) as f32 / i16::MAX as f32;
        info!(
            samples = pcm_i16.len(),
            peak = capture_peak,
            "captured utterance"
        );

        let mut vad = VoiceActivityDetector::default_detector();
        let trim = vad
            .trim_silence(&pcm_i16)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        self.latency.mark_vad_done();
        info!(trimmed_samples = trim.trimmed.len(), "vad trim complete");

        let settings = self.settings.read().await.clone();

        let output_mode = active_app_id
            .as_ref()
            .and_then(|id| {
                settings
                    .app_profiles
                    .iter()
                    .find(|p| &p.app_id == id)
                    .map(|p| p.output_mode)
            })
            .unwrap_or(OutputMode::Balanced);

        if trim.trimmed.is_empty() {
            return Ok(self.error_result("No speech detected — try speaking closer to the mic"));
        }

        self.state = DictationState::Transcribing;
        self.latency.mark_transcribe_start();

        let model = format!("whisper/{}", settings.whisper.model_id);
        let initial_prompt = whisper_initial_prompt(&settings.dictionary);
        let raw_transcript = match self
            .whisper
            .transcribe_pcm_i16(&trim.trimmed, SAMPLE_RATE, Some(&initial_prompt))
            .await
        {
            Ok(t) => t,
            Err(e) => return Ok(self.error_result(format!("Transcription failed: {e}"))),
        };

        self.latency.mark_transcribe_done();

        // Silence transcribes to non-speech markers, which are stripped to
        // nothing upstream — surface that as "no speech" instead of pasting.
        if raw_transcript.trim().is_empty() {
            return Ok(self.error_result("No speech detected — check the mic is picking you up"));
        }

        let estimated_usd = 0.0_f32;
        let provider_id = "local-whisper".to_string();
        let is_self_hosted = true;

        let mut transcript_text = apply_rules(&raw_transcript, output_mode);
        transcript_text = apply_snippets(&transcript_text, &settings.snippets);
        transcript_text = apply_dictionary(&transcript_text, &settings.dictionary);

        if settings.rewrite_enabled {
            self.state = DictationState::Cleaning;
            let rewrite_client = Self::resolve_client(&settings.rewrite_provider).await;
            let system_prompt =
                system_prompt_for_mode(output_mode, &settings.rewrite_prompt, &settings.dictionary);
            match rewrite_client
                .chat_rewrite(
                    &transcript_text,
                    &system_prompt,
                    &settings.rewrite_provider.model,
                )
                .await
            {
                Ok(rewritten) if !rewritten.trim().is_empty() => {
                    transcript_text = rewritten;
                }
                Ok(_) => warn!("rewrite returned empty, keeping local transcript"),
                Err(e) => warn!("rewrite failed: {e}"),
            }
        }

        // The rewrite model is given vocabulary context, but a confirmed exact
        // user replacement must win if the model changes it back.
        transcript_text = apply_dictionary(&transcript_text, &settings.dictionary);

        // Final sentence-level cleanup: capitalization, terminal punctuation,
        // question detection, and a trailing space between dictations. Runs
        // after the (optional) AI rewrite so it also cleans up local-only output
        // and normalizes the AI result's spacing.
        transcript_text = finalize_text(&transcript_text, output_mode);

        self.state = DictationState::Inserting;
        self.latency.mark_insert_start();

        let insert_result = match self
            .inserter
            .insert(&transcript_text, settings.clipboard_restore)
            .await
        {
            Ok(r) => {
                self.state = DictationState::Done;
                r
            }
            Err(e) => {
                warn!("insert failed: {e}, copying to clipboard");
                let _ = self.inserter.copy_only(&transcript_text).await;
                self.state = DictationState::Copied;
                voxflow_insert::InsertResult {
                    success: false,
                    method: voxflow_insert::InsertMethod::CopyOnly,
                    restored_clipboard: false,
                }
            }
        };

        self.latency.mark_insert_done();
        let report = self.latency.report();
        info!(?report, "dictation latency");

        if settings.privacy.save_history {
            let entry = new_entry(
                transcript_text.trim().to_string(),
                &provider_id,
                &model,
                trim.raw_duration_secs,
                trim.trimmed_duration_secs,
                active_app_id.clone(),
                estimated_usd,
            );
            let _ = self.history.insert(&entry);
            let _ = self.history.enforce_limit(settings.max_history_entries());
        }

        self.monthly_usage.add_record(UsageRecord {
            date: chrono::Local::now().date_naive(),
            duration_raw_secs: trim.raw_duration_secs,
            duration_billable_secs: trim.trimmed_duration_secs,
            provider: provider_id.clone(),
            model: model.clone(),
            estimated_usd,
            was_self_hosted: is_self_hosted,
        });

        let now = chrono::Local::now();
        let _ = self.history.save_monthly_usage(
            now.month(),
            now.year(),
            &serde_json::to_string(&self.monthly_usage)?,
        );

        let latency_ms = report.total_ms.unwrap_or(0);
        let event = StateEvent {
            state: self.state,
            ui_state: self.state.into(),
            message: None,
            transcript_preview: Some(transcript_text.chars().take(80).collect()),
        };

        Ok((
            event,
            DictationResult {
                text: transcript_text,
                state: self.state,
                insert_result: Some(insert_result),
                latency_ms,
            },
        ))
    }

    pub async fn settings(&self) -> Settings {
        self.settings.read().await.clone()
    }

    pub async fn update_settings(&self, settings: Settings) -> Result<()> {
        voxflow_config::save_settings(&settings)?;
        *self.settings.write().await = settings;
        Ok(())
    }

    pub async fn correct_history_entry(
        &mut self,
        id: &str,
        corrected_text: &str,
    ) -> Result<HistoryCorrectionResult> {
        let corrected_text = corrected_text.trim();
        if corrected_text.is_empty() {
            anyhow::bail!("corrected text cannot be empty");
        }

        let entry = self
            .history
            .get(id)?
            .ok_or_else(|| anyhow::anyhow!("history entry not found"))?;
        let settings = self.settings.read().await.clone();
        let suggestion = vocabulary_suggestion(&entry.text, corrected_text).filter(|candidate| {
            !settings.dictionary.iter().any(|entry| {
                entry.term.eq_ignore_ascii_case(&candidate.term)
                    && entry
                        .replacement
                        .as_deref()
                        .is_some_and(|value| value.eq_ignore_ascii_case(&candidate.replacement))
            }) && !settings
                .vocabulary_suggestion_dismissals
                .iter()
                .any(|dismissal| {
                    dismissal.term.eq_ignore_ascii_case(&candidate.term)
                        && dismissal
                            .replacement
                            .eq_ignore_ascii_case(&candidate.replacement)
                })
        });
        self.history.update_text(id, corrected_text)?;
        self.inserter
            .copy_only(corrected_text)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(HistoryCorrectionResult {
            entry: voxflow_history::HistoryEntry {
                text: corrected_text.to_string(),
                ..entry
            },
            suggestion,
        })
    }

    /// Returns a vocabulary suggestion for an externally observed, short-lived
    /// edit session. The caller owns the platform-specific observation; core
    /// only applies the same conservative eligibility and suppression rules as
    /// the History correction flow.
    pub async fn vocabulary_suggestion_for_edit(
        &self,
        original: &str,
        corrected: &str,
    ) -> Option<VocabularySuggestion> {
        let settings = self.settings.read().await;
        vocabulary_suggestion(original, corrected).filter(|candidate| {
            !settings.dictionary.iter().any(|entry| {
                entry.term.eq_ignore_ascii_case(&candidate.term)
                    && entry
                        .replacement
                        .as_deref()
                        .is_some_and(|value| value.eq_ignore_ascii_case(&candidate.replacement))
            }) && !settings
                .vocabulary_suggestion_dismissals
                .iter()
                .any(|dismissal| {
                    dismissal.term.eq_ignore_ascii_case(&candidate.term)
                        && dismissal
                            .replacement
                            .eq_ignore_ascii_case(&candidate.replacement)
                })
        })
    }

    pub async fn accept_vocabulary_suggestion(
        &mut self,
        suggestion: VocabularySuggestion,
    ) -> Result<()> {
        let term = suggestion.term.trim();
        let replacement = suggestion.replacement.trim();
        if !is_valid_vocabulary_phrase(term) || !is_valid_vocabulary_phrase(replacement) {
            anyhow::bail!("invalid vocabulary suggestion");
        }

        let mut settings = self.settings.read().await.clone();
        let duplicate = settings.dictionary.iter().any(|entry| {
            entry.term.eq_ignore_ascii_case(term)
                && entry
                    .replacement
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(replacement))
        });
        if !duplicate {
            settings.dictionary.push(DictionaryEntry {
                term: term.to_string(),
                replacement: Some(replacement.to_string()),
            });
        }
        settings
            .vocabulary_suggestion_dismissals
            .retain(|dismissal| {
                !(dismissal.term.eq_ignore_ascii_case(term)
                    && dismissal.replacement.eq_ignore_ascii_case(replacement))
            });
        voxflow_config::save_settings(&settings)?;
        *self.settings.write().await = settings;
        Ok(())
    }

    pub async fn dismiss_vocabulary_suggestion(
        &mut self,
        suggestion: VocabularySuggestion,
    ) -> Result<()> {
        let term = suggestion.term.trim();
        let replacement = suggestion.replacement.trim();
        if term.is_empty() || replacement.is_empty() {
            anyhow::bail!("invalid vocabulary suggestion");
        }

        let mut settings = self.settings.read().await.clone();
        let exists = settings
            .vocabulary_suggestion_dismissals
            .iter()
            .any(|dismissal| {
                dismissal.term.eq_ignore_ascii_case(term)
                    && dismissal.replacement.eq_ignore_ascii_case(replacement)
            });
        if !exists {
            settings
                .vocabulary_suggestion_dismissals
                .push(VocabularySuggestionDismissal {
                    term: term.to_string(),
                    replacement: replacement.to_string(),
                });
            // Retain the most recent dismissals without letting a settings file
            // grow unbounded from one-off edits.
            const MAX_DISMISSALS: usize = 100;
            if settings.vocabulary_suggestion_dismissals.len() > MAX_DISMISSALS {
                let excess = settings.vocabulary_suggestion_dismissals.len() - MAX_DISMISSALS;
                settings.vocabulary_suggestion_dismissals.drain(0..excess);
            }
            voxflow_config::save_settings(&settings)?;
            *self.settings.write().await = settings;
        }
        Ok(())
    }

    pub async fn restore_vocabulary_suggestion(
        &mut self,
        suggestion: VocabularySuggestion,
    ) -> Result<()> {
        let mut settings = self.settings.read().await.clone();
        let before = settings.vocabulary_suggestion_dismissals.len();
        settings
            .vocabulary_suggestion_dismissals
            .retain(|dismissal| {
                !(dismissal.term.eq_ignore_ascii_case(&suggestion.term)
                    && dismissal
                        .replacement
                        .eq_ignore_ascii_case(&suggestion.replacement))
            });
        if settings.vocabulary_suggestion_dismissals.len() != before {
            voxflow_config::save_settings(&settings)?;
            *self.settings.write().await = settings;
        }
        Ok(())
    }

    pub fn cost_dashboard(&self, settings: &Settings) -> voxflow_cost::CostDashboard {
        build_dashboard(
            &self.monthly_usage,
            &format!("{:?}", settings.transcription_provider.kind),
            &settings.transcription_provider.model,
            settings.cost_control.monthly_minute_cap,
            settings.cost_control.monthly_spend_cap_usd,
            &settings.cost_control.warn_at_percent,
        )
    }

    pub fn cap_reached(&self, settings: &Settings) -> bool {
        cap_reached(
            &self.monthly_usage,
            settings.cost_control.monthly_minute_cap,
            settings.cost_control.monthly_spend_cap_usd,
        )
    }

    pub fn list_history(&self, limit: u32) -> Result<Vec<voxflow_history::HistoryEntry>> {
        Ok(self.history.list(limit)?)
    }

    pub fn export_history_json(&self, limit: u32) -> Result<String> {
        Ok(self.history.export_json(limit)?)
    }

    pub fn export_history_csv(&self, limit: u32) -> Result<String> {
        Ok(self.history.export_csv(limit)?)
    }

    pub async fn paste_text(&self, text: &str) -> Result<InsertResult> {
        let settings = self.settings.read().await.clone();
        self.inserter
            .insert(text, settings.clipboard_restore)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn vocabulary_suggestion(original: &str, corrected: &str) -> Option<VocabularySuggestion> {
    let original_words: Vec<&str> = original.split_whitespace().collect();
    let corrected_words: Vec<&str> = corrected.split_whitespace().collect();
    if original_words.is_empty() || corrected_words.is_empty() {
        return None;
    }

    let mut prefix = 0;
    while prefix < original_words.len()
        && prefix < corrected_words.len()
        && comparable_word(original_words[prefix]) == comparable_word(corrected_words[prefix])
    {
        prefix += 1;
    }

    let mut original_end = original_words.len();
    let mut corrected_end = corrected_words.len();
    while original_end > prefix
        && corrected_end > prefix
        && comparable_word(original_words[original_end - 1])
            == comparable_word(corrected_words[corrected_end - 1])
    {
        original_end -= 1;
        corrected_end -= 1;
    }

    let term = phrase_without_edge_punctuation(&original_words[prefix..original_end]);
    let replacement = phrase_without_edge_punctuation(&corrected_words[prefix..corrected_end]);
    // A complete multi-word rewrite has no unchanged context proving this was
    // a spelling fix rather than normal editing. Only learn whole-utterance
    // replacements when both sides are a single token.
    let replaces_entire_utterance = prefix == 0
        && original_end == original_words.len()
        && corrected_end == corrected_words.len();
    if !is_valid_vocabulary_phrase(&term)
        || !is_valid_vocabulary_phrase(&replacement)
        || comparable_word(&term) == comparable_word(&replacement)
        || !looks_vocabulary_like(&term, &replacement)
        || (replaces_entire_utterance && (original_words.len() != 1 || corrected_words.len() != 1))
    {
        return None;
    }

    Some(VocabularySuggestion { term, replacement })
}

fn comparable_word(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn phrase_without_edge_punctuation(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_valid_vocabulary_phrase(phrase: &str) -> bool {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    !words.is_empty()
        && words.len() <= 4
        && phrase.chars().count() <= 48
        && phrase
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.')
        && phrase.chars().any(|c| c.is_alphanumeric())
}

fn looks_vocabulary_like(term: &str, replacement: &str) -> bool {
    term.chars()
        .chain(replacement.chars())
        .any(|c| c.is_uppercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod correction_tests {
    use super::*;

    #[test]
    fn suggests_a_small_distinctive_phrase_replacement() {
        assert_eq!(
            vocabulary_suggestion(
                "I want to submit this app for Shipper10.",
                "I want to submit this app for Shipaton."
            ),
            Some(VocabularySuggestion {
                term: "Shipper10".into(),
                replacement: "Shipaton".into(),
            })
        );
    }

    #[test]
    fn ignores_punctuation_and_sentence_rewrites() {
        assert!(vocabulary_suggestion("Hello world", "Hello, world!").is_none());
        assert!(vocabulary_suggestion("Ship it today", "Please submit this tomorrow").is_none());
    }

    #[test]
    fn ignores_common_word_replacements() {
        assert!(vocabulary_suggestion("I went home", "I go home").is_none());
    }
}
