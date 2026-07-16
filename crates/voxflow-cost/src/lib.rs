use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const OPENAI_MINI_USD_PER_MIN: f32 = 0.003;
pub const OPENAI_ACCURATE_USD_PER_MIN: f32 = 0.006;
pub const INR_PER_USD: f32 = 83.0;

#[derive(Debug, Error)]
pub enum CostError {
    #[error("monthly cap reached")]
    CapReached,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageRecord {
    pub date: NaiveDate,
    pub duration_raw_secs: f32,
    pub duration_billable_secs: f32,
    pub provider: String,
    pub model: String,
    pub estimated_usd: f32,
    pub was_self_hosted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonthlyUsage {
    pub month: u32,
    pub year: i32,
    pub records: Vec<UsageRecord>,
}

impl MonthlyUsage {
    pub fn current() -> Self {
        let now = Local::now();
        Self {
            month: now.month(),
            year: now.year(),
            records: Vec::new(),
        }
    }

    pub fn billable_minutes(&self) -> f32 {
        self.records
            .iter()
            .map(|r| r.duration_billable_secs)
            .sum::<f32>()
            / 60.0
    }

    pub fn total_usd(&self) -> f32 {
        self.records.iter().map(|r| r.estimated_usd).sum()
    }

    pub fn self_hosted_percentage(&self) -> f32 {
        if self.records.is_empty() {
            return 100.0;
        }
        let self_hosted = self.records.iter().filter(|r| r.was_self_hosted).count() as f32;
        (self_hosted / self.records.len() as f32) * 100.0
    }

    pub fn cloud_percentage(&self) -> f32 {
        100.0 - self.self_hosted_percentage()
    }

    pub fn add_record(&mut self, record: UsageRecord) {
        self.records.push(record);
    }

    pub fn projected_monthly_usd(&self) -> f32 {
        let now = Local::now();
        let day = now.day().max(1) as f32;
        let days_in_month = now
            .date_naive()
            .with_day(1)
            .and_then(|d| d.with_month(now.month() % 12 + 1))
            .map(|d| (d - chrono::Duration::days(1)).day())
            .unwrap_or(30) as f32;
        let spent = self.total_usd();
        if day <= 1.0 {
            return spent;
        }
        (spent / day) * days_in_month
    }

    pub fn average_daily_usd(&self) -> f32 {
        let now = Local::now();
        let day = now.day().max(1) as f32;
        self.total_usd() / day
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDashboard {
    pub minutes_transcribed: f32,
    pub estimated_usd: f32,
    pub estimated_inr: f32,
    pub current_provider: String,
    pub current_model: String,
    pub average_daily_usd: f32,
    pub projected_monthly_usd: f32,
    pub projected_monthly_inr: f32,
    pub self_hosted_percentage: f32,
    pub cloud_percentage: f32,
    pub cap_warnings: Vec<String>,
}

pub fn estimate_openai_cost_usd(model: &str, billable_secs: f32) -> f32 {
    let minutes = billable_secs / 60.0;
    let rate = if model.contains("gpt-4o-transcribe") && !model.contains("mini") {
        OPENAI_ACCURATE_USD_PER_MIN
    } else {
        OPENAI_MINI_USD_PER_MIN
    };
    minutes * rate
}

pub fn usd_to_inr(usd: f32) -> f32 {
    usd * INR_PER_USD
}

pub fn check_caps(
    usage: &MonthlyUsage,
    minute_cap: Option<f32>,
    spend_cap_usd: Option<f32>,
    warn_at: &[u8],
) -> Vec<String> {
    let mut warnings = Vec::new();
    let minutes = usage.billable_minutes();
    let spent = usage.total_usd();

    if let Some(cap) = minute_cap {
        for pct in warn_at {
            let threshold = cap * (*pct as f32 / 100.0);
            if minutes >= threshold {
                warnings.push(format!(
                    "Cloud minutes at {pct}% of cap ({minutes:.1}/{cap:.0} min)"
                ));
            }
        }
    }

    if let Some(cap) = spend_cap_usd {
        for pct in warn_at {
            let threshold = cap * (*pct as f32 / 100.0);
            if spent >= threshold {
                warnings.push(format!("Spend at {pct}% of cap (${spent:.2}/${cap:.2})"));
            }
        }
    }

    warnings.sort();
    warnings.dedup();
    warnings
}

pub fn cap_reached(
    usage: &MonthlyUsage,
    minute_cap: Option<f32>,
    spend_cap_usd: Option<f32>,
) -> bool {
    if let Some(cap) = minute_cap {
        if usage.billable_minutes() >= cap {
            return true;
        }
    }
    if let Some(cap) = spend_cap_usd {
        if usage.total_usd() >= cap {
            return true;
        }
    }
    false
}

pub fn build_dashboard(
    usage: &MonthlyUsage,
    provider: &str,
    model: &str,
    minute_cap: Option<f32>,
    spend_cap_usd: Option<f32>,
    warn_at: &[u8],
) -> CostDashboard {
    let estimated_usd = usage.total_usd();
    CostDashboard {
        minutes_transcribed: usage.billable_minutes(),
        estimated_usd,
        estimated_inr: usd_to_inr(estimated_usd),
        current_provider: provider.into(),
        current_model: model.into(),
        average_daily_usd: usage.average_daily_usd(),
        projected_monthly_usd: usage.projected_monthly_usd(),
        projected_monthly_inr: usd_to_inr(usage.projected_monthly_usd()),
        self_hosted_percentage: usage.self_hosted_percentage(),
        cloud_percentage: usage.cloud_percentage(),
        cap_warnings: check_caps(usage, minute_cap, spend_cap_usd, warn_at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_mini_cost() {
        let cost = estimate_openai_cost_usd("gpt-4o-mini-transcribe", 60.0);
        assert!((cost - 0.003).abs() < 0.0001);
    }

    #[test]
    fn cap_detection() {
        let mut usage = MonthlyUsage::current();
        usage.add_record(UsageRecord {
            date: Local::now().date_naive(),
            duration_raw_secs: 3600.0,
            duration_billable_secs: 3600.0,
            provider: "openai".into(),
            model: "gpt-4o-mini-transcribe".into(),
            estimated_usd: 0.18,
            was_self_hosted: false,
        });
        assert!(!cap_reached(&usage, Some(100.0), Some(5.0)));
        assert!(cap_reached(&usage, Some(30.0), None));
    }
}
