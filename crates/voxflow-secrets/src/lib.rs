//! OS-native secret storage for VoxFlow: macOS Keychain / Windows Credential
//! Manager via the `keyring` crate. Provider API keys are looked up by a
//! stable `key_ref` (e.g. "transcription", "rewrite", or a per-custom-endpoint
//! UUID) and are never stored in the plaintext settings JSON.

use thiserror::Error;

const SERVICE: &str = "com.maskedsyntax.voxflow";

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

pub fn set_secret(key_ref: &str, secret: &str) -> Result<(), SecretError> {
    let entry = keyring::Entry::new(SERVICE, key_ref)?;
    entry.set_password(secret)?;
    Ok(())
}

pub fn get_secret(key_ref: &str) -> Result<Option<String>, SecretError> {
    let entry = keyring::Entry::new(SERVICE, key_ref)?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_secret(key_ref: &str) -> Result<(), SecretError> {
    let entry = keyring::Entry::new(SERVICE, key_ref)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
