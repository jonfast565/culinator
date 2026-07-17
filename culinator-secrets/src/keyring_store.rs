use culinator_models::{ApplicationError, SecretStore};
use keyring::Entry;

use crate::CULINATOR_SERVICE;

#[derive(Debug, Clone, Default)]
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    pub fn new() -> Self {
        Self
    }

    fn entry(key: &str) -> Result<Entry, ApplicationError> {
        Entry::new(CULINATOR_SERVICE, key)
            .map_err(|e| ApplicationError::Persistence(format!("keyring entry failed: {e}")))
    }

    pub fn is_available() -> bool {
        match Self::entry("__culinator_probe__") {
            Ok(entry) => match entry.get_password() {
                Ok(_) | Err(keyring::Error::NoEntry) => true,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

impl SecretStore for KeyringSecretStore {
    fn get_secret(&self, key: &str) -> Result<Option<String>, ApplicationError> {
        match Self::entry(key)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(ApplicationError::Persistence(format!(
                "keyring read failed: {error}"
            ))),
        }
    }

    fn set_secret(&self, key: &str, value: &str) -> Result<(), ApplicationError> {
        Self::entry(key)?
            .set_password(value)
            .map_err(|e| ApplicationError::Persistence(format!("keyring write failed: {e}")))
    }

    fn delete_secret(&self, key: &str) -> Result<(), ApplicationError> {
        match Self::entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(ApplicationError::Persistence(format!(
                "keyring delete failed: {error}"
            ))),
        }
    }
}
