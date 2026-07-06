use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;
use voxflow_config::{QualityMode, Settings};
use voxflow_cost::{cap_reached, MonthlyUsage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteTarget {
    Local,
    OpenAiMini,
    OpenAiAccurate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub target: RouteTarget,
    pub model: String,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("offline mode requires local provider")]
    OfflineOnly,
    #[error("monthly cap reached — local only")]
    CapReached,
}

pub struct ProviderRouter;

impl ProviderRouter {
    pub fn decide(
        settings: &Settings,
        usage: &MonthlyUsage,
        duration_after_vad_secs: f32,
        offline: bool,
        active_app_id: Option<&str>,
        noisy: bool,
    ) -> Result<RouteDecision, RouterError> {
        if offline || settings.quality_mode == QualityMode::Offline {
            return Ok(RouteDecision {
                target: RouteTarget::Local,
                model: settings.local_model.clone(),
                reason: "offline mode".into(),
            });
        }

        let cap_hit = cap_reached(
            usage,
            settings.cost_control.monthly_minute_cap,
            settings.cost_control.monthly_spend_cap_usd,
        );
        if cap_hit && settings.cost_control.auto_local_after_cap {
            return Ok(RouteDecision {
                target: RouteTarget::Local,
                model: settings.local_model.clone(),
                reason: "monthly cap reached".into(),
            });
        }

        if let Some(app_id) = active_app_id {
            if let Some(profile) = settings.app_profiles.iter().find(|p| p.app_id == app_id) {
                if profile.disable_cloud {
                    return Ok(RouteDecision {
                        target: RouteTarget::Local,
                        model: settings.local_model.clone(),
                        reason: format!("cloud disabled for {}", profile.name),
                    });
                }
            }
        }

        if settings.privacy.sensitive_app_blocklist.iter().any(|id| {
            active_app_id.map(|a| a == id.as_str()).unwrap_or(false)
        }) {
            return Err(RouterError::CapReached);
        }

        let decision = match settings.quality_mode {
            QualityMode::Offline => RouteDecision {
                target: RouteTarget::Local,
                model: settings.local_model.clone(),
                reason: "offline".into(),
            },
            QualityMode::Economy => RouteDecision {
                target: if duration_after_vad_secs < 5.0 {
                    RouteTarget::Local
                } else {
                    RouteTarget::OpenAiMini
                },
                model: if duration_after_vad_secs < 5.0 {
                    settings.local_model.clone()
                } else {
                    "gpt-4o-mini-transcribe".into()
                },
                reason: "economy mode".into(),
            },
            QualityMode::Accurate | QualityMode::Balanced if noisy => RouteDecision {
                target: RouteTarget::OpenAiAccurate,
                model: "gpt-4o-transcribe".into(),
                reason: "noisy audio — accurate model".into(),
            },
            QualityMode::Accurate => RouteDecision {
                target: RouteTarget::OpenAiAccurate,
                model: "gpt-4o-transcribe".into(),
                reason: "accurate mode".into(),
            },
            QualityMode::Hybrid | QualityMode::Balanced => {
                if duration_after_vad_secs < 5.0 {
                    RouteDecision {
                        target: RouteTarget::Local,
                        model: settings.local_model.clone(),
                        reason: "short utterance — local first".into(),
                    }
                } else {
                    RouteDecision {
                        target: RouteTarget::OpenAiMini,
                        model: settings.openai_model.clone(),
                        reason: "hybrid cloud route".into(),
                    }
                }
            }
        };

        debug!(?decision, "provider route decided");
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_short_goes_local() {
        let settings = Settings::default();
        let usage = MonthlyUsage::current();
        let d = ProviderRouter::decide(&settings, &usage, 3.0, false, None, false).unwrap();
        assert_eq!(d.target, RouteTarget::Local);
    }

    #[test]
    fn hybrid_long_goes_cloud() {
        let settings = Settings::default();
        let usage = MonthlyUsage::current();
        let d = ProviderRouter::decide(&settings, &usage, 10.0, false, None, false).unwrap();
        assert_eq!(d.target, RouteTarget::OpenAiMini);
    }
}
