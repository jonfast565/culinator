use super::*;
use culinator_models::SecretStore;
use tempfile::tempdir;

#[test]
fn encrypted_file_secret_store_round_trip() {
    let dir = tempdir().unwrap();
    let store = EncryptedFileSecretStore::new(dir.path().join("secrets.enc"));

    assert_eq!(store.get_secret(OPENAI_API_KEY).unwrap(), None);

    store
        .set_secret(OPENAI_API_KEY, "sk-test-key")
        .expect("set secret");
    assert_eq!(
        store.get_secret(OPENAI_API_KEY).unwrap(),
        Some("sk-test-key".to_owned())
    );

    let reloaded = EncryptedFileSecretStore::new(dir.path().join("secrets.enc"));
    assert_eq!(
        reloaded.get_secret(OPENAI_API_KEY).unwrap(),
        Some("sk-test-key".to_owned())
    );

    store.delete_secret(OPENAI_API_KEY).expect("delete secret");
    assert_eq!(store.get_secret(OPENAI_API_KEY).unwrap(), None);
}

#[test]
fn encrypted_file_secret_store_supports_multiple_keys() {
    let dir = tempdir().unwrap();
    let store = EncryptedFileSecretStore::new(dir.path().join("secrets.enc"));

    store.set_secret("one", "alpha").unwrap();
    store.set_secret("two", "beta").unwrap();

    assert_eq!(store.get_secret("one").unwrap(), Some("alpha".to_owned()));
    assert_eq!(store.get_secret("two").unwrap(), Some("beta".to_owned()));
}
