use crate::ApplicationError;

/// Secure storage for sensitive settings (API keys). Implementations may use
/// the OS keychain or an encrypted local file fallback.
pub trait SecretStore: Send + Sync {
    fn get_secret(&self, key: &str) -> Result<Option<String>, ApplicationError>;
    fn set_secret(&self, key: &str, value: &str) -> Result<(), ApplicationError>;
    fn delete_secret(&self, key: &str) -> Result<(), ApplicationError>;
}
