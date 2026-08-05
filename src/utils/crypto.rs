use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use russh::keys::ssh_key::sha2::{Digest, Sha256};

const NONCE_SIZE: usize = 12;

/// Encrypts a password for local storage (such as an OS keyring or config file).
///
/// # Security Limitations
/// * **Device-Bound:** The encryption key is derived dynamically using the current
///   OS username (`whoami`). Consequently, the resulting ciphertext **cannot** be
///   decrypted on a different machine or under a different user account.
/// * **Obfuscation, Not Zero-Trust:** Because the key generation relies entirely
///   on local, unsecret OS identifiers, anyone with root/administrator access
///   on the machine can reproduce the key and decrypt the payload.
///   This is intended solely to prevent plaintext exposure at rest, not to
///   protect against a compromised host.
pub fn encrypt_password(password: &str) -> Result<String> {
    let key = get_dynamic_key()?;
    let cipher = Aes256Gcm::new(&key.into());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

/// Decrypts a password that was previously encrypted via `encrypt_password`
/// on this same machine and user account.
pub fn decrypt_password(encrypted_password: &str) -> Result<String> {
    let key = get_dynamic_key()?;
    let cipher = Aes256Gcm::new(&key.into());
    let decoded_bytes = general_purpose::STANDARD.decode(encrypted_password)?;

    if decoded_bytes.len() < NONCE_SIZE {
        return Err(anyhow::anyhow!("Encrypted data is too short"));
    }

    let (nonce_bytes, ciphertext_bytes) = decoded_bytes.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext_bytes)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(decrypted_bytes)?)
}

fn get_dynamic_key() -> Result<[u8; 32]> {
    let username = whoami::username()?;
    let mut hasher = Sha256::new();
    // Typos are intentional :P
    hasher.update(b"kNoT-NoNcRiPtOgRpHic-sUlt-:3 CATs mEoW :3");
    hasher.update(username.as_bytes());
    let result = hasher.finalize();
    let key: [u8; 32] = result.into();
    Ok(key)
}
