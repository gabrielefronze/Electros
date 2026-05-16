use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

const SERVICE: &str = "app.elemento.cloud.electros";
const USER: &str = "safe-storage-master";

fn master_key_bytes() -> Option<Vec<u8>> {
    let entry = keyring::Entry::new(SERVICE, USER).ok()?;
    if let Ok(existing) = entry.get_password() {
        return Some(Sha256::digest(existing.as_bytes()).to_vec());
    }
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    let encoded = hex::encode(key);
    entry.set_password(&encoded).ok()?;
    Some(Sha256::digest(encoded.as_bytes()).to_vec())
}

pub fn is_encryption_available() -> bool {
    master_key_bytes().is_some()
}

pub fn encrypt_string(value: &str, refuse_unsafe: bool) -> Result<serde_json::Value, String> {
    let Some(key_bytes) = master_key_bytes() else {
        if refuse_unsafe {
            return Ok(serde_json::Value::Bool(false));
        }
        return Ok(serde_json::Value::String(value.to_string()));
    };

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut packed = nonce_bytes.to_vec();
    packed.extend(ciphertext);
    Ok(serde_json::Value::String(B64.encode(packed)))
}

pub fn decrypt_string(value: &str) -> Result<serde_json::Value, String> {
    let Some(key_bytes) = master_key_bytes() else {
        return Ok(serde_json::Value::Bool(false));
    };
    let packed = B64.decode(value).map_err(|e| e.to_string())?;
    if packed.len() < 13 {
        return Err("Invalid ciphertext".into());
    }
    let (nonce_bytes, ct) = packed.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ct)
        .map_err(|e| e.to_string())?;
    let s = String::from_utf8(plaintext).map_err(|e| e.to_string())?;
    Ok(serde_json::Value::String(s))
}
