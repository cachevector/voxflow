use crate::events::{DictationState, StateEvent};
use crate::latency::LatencyTracker;
use anyhow::{Context, Result};
use chrono::Datelike;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use voxflow_audio::AudioCapture;
use voxflow_config::{OutputMode, Settings};
use voxflow_cost::{
    build_dashboard, cap_reached, estimate_openai_cost_usd, MonthlyUsage, UsageRecord,
};
use voxflow_history::{HistoryStore, new_entry};
use voxflow_insert::{InsertionBridge, InsertResult};
use voxflow_router::{ProviderRouter, RouteTarget};
use voxflow_transcribe::{
    apply_dictionary, apply_snippets, AudioSegment, CleanupEngine, LocalWhisperTranscriber,
    OpenAiTranscriber, TranscribeOpts, TranscriptionProvider,
};
use voxflow_vad::{VoiceActivityDetector, SAMPLE_RATE};

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
    history: HistoryStore,
    monthly_usage: MonthlyUsage,
    latency: LatencyTracker,
    state: DictationState,
    offline: bool,
}

impl DictationPipeline {
    pub fn new(settings: Settings, inserter: Arc<InsertionBridge>) -> Result<Self> {
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
            history,
            monthly_usage,
            latency: LatencyTracker::new(),
            state: DictationState::Idle,
            offline: false,
        })
    }

    pub fn state(&self) -> DictationState {
        self.state
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    pub async fn prewarm(&mut self) -> Result<()> {
        let device = self.settings.read().await.microphone_device.clone();
        let capture = AudioCapture::prewarm(device.as_deref()).context("prewarm audio")?;
        self.capture = Some(capture);
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
                    return StateEvent::new(
                        self.state,
                        Some(format!("Microphone error: {e}")),
                    );
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

    #[allow(unused_assignments)]
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

        let mut vad = VoiceActivityDetector::default_detector();
        let trim = vad
            .trim_silence(&pcm_i16)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        self.latency.mark_vad_done();

        let settings = self.settings.read().await.clone();
        let route = ProviderRouter::decide(
            &settings,
            &self.monthly_usage,
            trim.trimmed_duration_secs,
            self.offline,
            active_app_id.as_deref(),
            false,
        );

        self.state = DictationState::Transcribing;
        self.latency.mark_transcribe_start();

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

        let segment = AudioSegment {
            pcm_i16: trim.trimmed.clone(),
            sample_rate: SAMPLE_RATE,
            duration_secs: trim.trimmed_duration_secs,
        };

        let opts = TranscribeOpts {
            model: settings.openai_model.clone(),
            language: None,
            prompt: None,
        };

        let mut transcript_text = String::new();
        let mut provider_id = String::new();
        let mut model_used = String::new();
        let mut was_local = false;
        let mut estimated_usd = 0.0f32;

        match route {
            Ok(decision) => {
                model_used = decision.model.clone();
                match decision.target {
                    RouteTarget::Local => {
                        was_local = true;
                        let local = LocalWhisperTranscriber::new(settings.local_model.clone());
                        provider_id = local.id().to_string();
                        match local.transcribe(&segment, &opts).await {
                            Ok(t) => transcript_text = t.text,
                            Err(e) => {
                                if let Some(key) = &settings.openai_api_key {
                                    let cloud = OpenAiTranscriber::new(key.clone());
                                    provider_id = cloud.id().to_string();
                                    was_local = false;
                                    let t = cloud
                                        .transcribe(&segment, &opts)
                                        .await
                                        .map_err(|ce| anyhow::anyhow!("{e}; cloud fallback: {ce}"))?;
                                    transcript_text = t.text;
                                    estimated_usd = estimate_openai_cost_usd(
                                        &model_used,
                                        trim.trimmed_duration_secs,
                                    );
                                } else {
                                    return Err(anyhow::anyhow!("{e}"));
                                }
                            }
                        }
                    }
                    RouteTarget::OpenAiMini | RouteTarget::OpenAiAccurate => {
                        let key = settings
                            .openai_api_key
                            .clone()
                            .context("OpenAI API key required")?;
                        let cloud = OpenAiTranscriber::new(key);
                        provider_id = cloud.id().to_string();
                        let mut cloud_opts = opts.clone();
                        cloud_opts.model = decision.model;
                        let t = cloud.transcribe(&segment, &cloud_opts).await?;
                        transcript_text = t.text;
                        model_used = t.model;
                        estimated_usd =
                            estimate_openai_cost_usd(&model_used, trim.trimmed_duration_secs);
                    }
                }
            }
            Err(e) => {
                self.state = DictationState::Error;
                return Ok((
                    StateEvent::new(self.state, Some(e.to_string())),
                    DictationResult {
                        text: String::new(),
                        state: self.state,
                        insert_result: None,
                        latency_ms: 0,
                    },
                ));
            }
        }

        self.latency.mark_transcribe_done();

        transcript_text = CleanupEngine::apply_rules(&transcript_text, output_mode);
        if settings.is_pro() {
            transcript_text = apply_snippets(&transcript_text, &settings.snippets);
            transcript_text = apply_dictionary(&transcript_text, &settings.dictionary);
        }

        if settings.cleanup_enabled && settings.is_pro() {
            if let Some(key) = &settings.openai_api_key {
                self.state = DictationState::Cleaning;
                let cleanup = CleanupEngine::new(key.clone());
                if let Ok(cleaned) = cleanup.cleanup(&transcript_text, &settings.cleanup_prompt).await {
                    transcript_text = cleaned;
                }
            }
        }

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
                transcript_text.clone(),
                &provider_id,
                &model_used,
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
            model: model_used.clone(),
            estimated_usd,
            was_local,
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
            "hybrid",
            &settings.openai_model,
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
}
