mod encrypted_file;
mod keyring_store;
mod resolve;

pub use encrypted_file::EncryptedFileSecretStore;
pub use keyring_store::KeyringSecretStore;
pub use resolve::resolve_secret_store;

/// OS keychain service name for Culinator secrets.
pub const CULINATOR_SERVICE: &str = "culinator";
/// Secret key name for the OpenAI API key.
pub const OPENAI_API_KEY: &str = "openai_api_key";

#[cfg(test)]
#[path = "encrypted_file/test.rs"]
mod encrypted_file_tests;
