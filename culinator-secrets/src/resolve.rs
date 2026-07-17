use culinator_models::SecretStore;
use std::{path::Path, sync::Arc};

use crate::{CULINATOR_SERVICE, EncryptedFileSecretStore, KeyringSecretStore};

pub fn resolve_secret_store(data_dir: &Path) -> Arc<dyn SecretStore> {
    if KeyringSecretStore::is_available() {
        eprintln!("Culinator secret store: OS keychain ({CULINATOR_SERVICE})");
        Arc::new(KeyringSecretStore::new())
    } else {
        let store = EncryptedFileSecretStore::new(data_dir.join("secrets.enc"));
        eprintln!(
            "Culinator secret store: encrypted file fallback ({})",
            store.path().display()
        );
        Arc::new(store)
    }
}
