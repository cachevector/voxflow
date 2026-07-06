use serde::{Deserialize, Serialize};
use voxflow_config::{LicenseTier, Settings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidation {
    pub valid: bool,
    pub tier: LicenseTier,
    pub message: String,
}

/// Offline-friendly license validation (Phase 3/5).
/// Pro keys: VFPRO- followed by 16 alphanumeric chars (checksum simplified).
pub fn validate_license_key(key: &str) -> LicenseValidation {
    let trimmed = key.trim().to_uppercase();
    if trimmed.is_empty() {
        return LicenseValidation {
            valid: false,
            tier: LicenseTier::Free,
            message: "No license key provided".into(),
        };
    }

    if trimmed.starts_with("VFBETA-") && trimmed.len() >= 12 {
        return LicenseValidation {
            valid: true,
            tier: LicenseTier::Beta,
            message: "Beta access granted".into(),
        };
    }

    if trimmed.starts_with("VFPRO-") && trimmed.len() >= 21 {
        let payload = &trimmed[6..];
        if payload.chars().all(|c| c.is_ascii_alphanumeric()) {
            return LicenseValidation {
                valid: true,
                tier: LicenseTier::Pro,
                message: "Pro lifetime license activated".into(),
            };
        }
    }

    LicenseValidation {
        valid: false,
        tier: LicenseTier::Free,
        message: "Invalid license key".into(),
    }
}

pub fn apply_license(settings: &mut Settings, key: &str) -> LicenseValidation {
    let result = validate_license_key(key);
    if result.valid {
        settings.license.key = Some(key.trim().to_string());
        settings.license.tier = result.tier;
        settings.license.validated_at = Some(chrono::Utc::now().to_rfc3339());
    }
    result
}

pub fn beta_invite_valid(code: &str) -> bool {
    code.trim().starts_with("INV-") && code.trim().len() >= 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pro_key_validates() {
        let v = validate_license_key("VFPRO-ABCD1234EFGH5678");
        assert!(v.valid);
        assert_eq!(v.tier, LicenseTier::Pro);
    }
}
