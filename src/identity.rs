use anyhow::{Context, Result};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub fn load_or_create(path: &Path) -> Result<SigningKey> {
    if path.exists() {
        let bytes = fs::read(path)?;
        let arr: [u8; 32] = bytes.as_slice().try_into().context("identity key must be exactly 32 bytes")?;
        return Ok(SigningKey::from_bytes(&arr));
    }
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let key = SigningKey::generate(&mut OsRng);
    let tmp = path.with_extension("key.tmp");
    fs::write(&tmp, key.to_bytes())?;
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(tmp, path)?;
    Ok(key)
}
pub fn public_id(path: &Path) -> Result<String> {
    let key = load_or_create(path)?;
    let public = key.verifying_key().to_bytes();
    let digest = Sha256::digest(public);
    Ok(hex::encode(digest))
}
pub fn public_key_b64(key: &SigningKey) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(key.verifying_key().to_bytes())
}
