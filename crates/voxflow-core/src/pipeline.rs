use crate::events::{DictationState, StateEvent};
use crate::latency::LatencyTracker;
use anyhow::{Context, Result};
use chrono::Datelike;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use voxflow_audio::AudioCapture;
use voxflow_config::{OutputMode, ProviderConfig, Settings};
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
            .map(|c| c.take_utterance_pcm_i16())
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
