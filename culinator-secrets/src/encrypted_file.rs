use chacha20poly1305::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use culinator_models::{ApplicationError, SecretStore};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const NONCE_LEN: usize = 12;
const APP_SALT: &[u8] = b"culinator-secrets-v1";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EncryptedBlob {
    nonce: String,
    ciphertext: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct SecretFile {
    #[serde(default)]
    secrets: HashMap<String, EncryptedBlob>,
}

#[derive(Debug, Clone)]
pub struct EncryptedFileSecretStore {
    path: PathBuf,
}

impl EncryptedFileSecretStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn encryption_key() -> Key {
        let mut material = Vec::new();
        if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            material.extend_from_slice(user.as_bytes());
        }
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            material.extend_from_slice(home.as_bytes());
        }
        if let Ok(exe) = std::env::current_exe() {
            material.extend_from_slice(exe.to_string_lossy().as_bytes());
        }
        material.extend_from_slice(APP_SALT);
        let digest = Sha256::digest(&material);
        *Key::from_slice(&digest)
    }

    fn read_file(&self) -> Result<SecretFile, ApplicationError> {
        if !self.path.exists() {
            return Ok(SecretFile::default());
        }
        let data = fs::read_to_string(&self.path)
            .map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        serde_json::from_str(&data)
            .map_err(|e| ApplicationError::Persistence(format!("invalid secrets file: {e}")))
    }

    fn write_file(&self, file: &SecretFile) -> Result<(), ApplicationError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        }
        let data = serde_json::to_vec_pretty(file)
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;
        let temp = self.path.with_extension("enc.tmp");
        fs::write(&temp, data).map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        fs::rename(&temp, &self.path).map_err(|e| ApplicationError::Persistence(e.to_string()))
    }

    fn encrypt(plaintext: &str) -> Result<EncryptedBlob, ApplicationError> {
        let cipher = ChaCha20Poly1305::new(&Self::encryption_key());
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| ApplicationError::Internal(format!("encryption failed: {e}")))?;
        Ok(EncryptedBlob {
            nonce: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, nonce_bytes),
            ciphertext: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                ciphertext,
            ),
        })
    }

    fn decrypt(blob: &EncryptedBlob) -> Result<String, ApplicationError> {
        let cipher = ChaCha20Poly1305::new(&Self::encryption_key());
        let nonce_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &blob.nonce)
                .map_err(|e| ApplicationError::Persistence(format!("invalid secret nonce: {e}")))?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(ApplicationError::Persistence(
                "invalid secret nonce length".into(),
            ));
        }
        let ciphertext =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &blob.ciphertext)
                .map_err(|e| {
                    ApplicationError::Persistence(format!("invalid secret ciphertext: {e}"))
                })?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| ApplicationError::Persistence(format!("decryption failed: {e}")))?;
        String::from_utf8(plaintext)
            .map_err(|e| ApplicationError::Persistence(format!("invalid secret utf-8: {e}")))
    }
}

impl SecretStore for EncryptedFileSecretStore {
    fn get_secret(&self, key: &str) -> Result<Option<String>, ApplicationError> {
        let file = self.read_file()?;
        match file.secrets.get(key) {
            Some(blob) => Ok(Some(Self::decrypt(blob)?)),
            None => Ok(None),
        }
    }

    fn set_secret(&self, key: &str, value: &str) -> Result<(), ApplicationError> {
        let mut file = self.read_file()?;
        file.secrets.insert(key.to_owned(), Self::encrypt(value)?);
        self.write_file(&file)
    }

    fn delete_secret(&self, key: &str) -> Result<(), ApplicationError> {
        let mut file = self.read_file()?;
        if file.secrets.remove(key).is_some() {
            self.write_file(&file)?;
        }
        Ok(())
    }
}
