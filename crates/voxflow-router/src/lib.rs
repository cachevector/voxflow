use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;
use voxflow_config::{QualityMode, Settings};
use voxflow_cost::{cap_reached, MonthlyUsage};

/// Which model tier to use on the configured transcription provider —
/// there's no separate "local" provider anymore, just a cheap vs. accurate
/// model on whatever endpoint (cloud or self-hosted) the user configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Cheap,
    Accurate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub tier: ModelTier,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("insertion blocked for sensitive app: {0}")]
    SensitiveApp(String),
}

pub struct ProviderRouter;

impl ProviderRouter {
    pub fn decide(
        settings: &Settings,
        usage: &MonthlyUsage,
        duration_after_vad_secs: f32,
        active_app_id: Option<&str>,
        noisy: bool,
    ) -> Result<RouteDecision, RouterError> {
        if let Some(app_id) = active_app_id {
            if settings
                .privacy
                .sensitive_app_blocklist
                .iter()
                .any(|id| id == app_id)
            {
                return Err(RouterError::SensitiveApp(app_id.to_string()));
            }
        }

        let cap_hit = cap_reached(
            usage,
            settings.cost_control.monthly_minute_cap,
            settings.cost_control.monthly_spend_cap_usd,
        );
        if cap_hit && settings.cost_control.auto_downgrade_after_cap {
            return Ok(RouteDecision {
                tier: ModelTier::Cheap,
                reason: "monthly cap reached — downgraded to cheap tier".into(),
            });
        }

        if let Some(app_id) = active_app_id {
            if let Some(profile) = settings.app_profiles.iter().find(|p| p.app_id == app_id) {
                if profile.disable_cloud {
                    return Ok(RouteDecision {
                        tier: ModelTier::Cheap,
                        reason: format!("cloud disabled for {}", profile.name),
                    });
                }
            }
        }

        let decision = match settings.quality_mode {
            QualityMode::Economy => RouteDecision {
                tier: ModelTier::Cheap,
                reason: "economy mode".into(),
            },
            QualityMode::Accurate => RouteDecision {
                tier: ModelTier::Accurate,
                reason: "accurate mode".into(),
            },
            QualityMode::Balanced if noisy => RouteDecision {
                tier: ModelTier::Accurate,
                reason: "noisy audio — accurate model".into(),
            },
            QualityMode::Balanced => RouteDecision {
                tier: ModelTier::Cheap,
                reason: "balanced mode".into(),
            },
            QualityMode::Hybrid => {
                if noisy || duration_after_vad_secs >= 60.0 {
                    RouteDecision {
                        tier: ModelTier::Accurate,
                        reason: "long/noisy audio — accurate model".into(),
                    }
                } else {
                    RouteDecision {
                        tier: ModelTier::Cheap,
                        reason: "hybrid — cheap model".into(),
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
    fn hybrid_short_goes_cheap() {
        let settings = Settings::default();
        let usage = MonthlyUsage::current();
        let d = ProviderRouter::decide(&settings, &usage, 3.0, None, false).unwrap();
        assert_eq!(d.tier, ModelTier::Cheap);
    }

    #[test]
    fn hybrid_long_goes_accurate() {
        let settings = Settings::default();
        let usage = MonthlyUsage::current();
        let d = ProviderRouter::decide(&settings, &usage, 90.0, None, false).unwrap();
        assert_eq!(d.tier, ModelTier::Accurate);
    }

    #[test]
    fn sensitive_app_is_blocked() {
        let settings = Settings::default();
        let usage = MonthlyUsage::current();
        let blocked_app = settings.privacy.sensitive_app_blocklist[0].clone();
        let result = ProviderRouter::decide(&settings, &usage, 3.0, Some(&blocked_app), false);
        assert!(matches!(result, Err(RouterError::SensitiveApp(_))));
    }
}
