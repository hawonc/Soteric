use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::{anyhow, Context, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use std::fs;
use std::path::Path;

pub struct Encrypter;

impl Encrypter {
    pub fn encrypt(files: &[String], key: &str) -> Result<()> {
        for file in files {
            let path = Path::new(file);
            let data = fs::read(path)
                .with_context(|| format!("failed to read file: {}", path.display()))?;

            let mut salt = [0u8; 16];
            let mut nonce_bytes = [0u8; 12];
            getrandom::fill(&mut salt)?;
            getrandom::fill(&mut nonce_bytes)?;

            let derived_key = Self::normalize_key(key, &salt);
            let key = Key::<Aes256Gcm>::try_from(&derived_key[..])
                .map_err(|_| anyhow!("invalid derived key length"))?;
            let cipher = Aes256Gcm::new(&key);

            let nonce = Nonce::try_from(&nonce_bytes[..])
                .map_err(|_| anyhow!("invalid nonce length"))?;

            let ciphertext = cipher
                .encrypt(&nonce, data.as_ref())
                .map_err(|e| anyhow!("Encryption error: {}", e))?;

            let mut output = salt.to_vec();
            output.extend_from_slice(&nonce_bytes);
            output.extend_from_slice(&ciphertext);

            fs::write(path, output)
                .with_context(|| format!("failed to write encrypted file: {}", path.display()))?;
        }
        Ok(())
    }

    pub fn decrypt(files: &[String], key: &str) -> Result<()> {
        for file in files {
            let path = Path::new(file);
            let data = fs::read(path).with_context(|| format!("failed to read {}", file))?;

            if data.len() < 28 {
                return Err(anyhow!("File {} is too small", file));
            }

            let (salt, rest) = data.split_at(16);
            let (nonce_bytes, ciphertext) = rest.split_at(12);

            let salt_arr: [u8; 16] = salt.try_into()?;
            let nonce_arr: [u8; 12] = nonce_bytes.try_into()?;

            let derived_key = Self::normalize_key(key, &salt_arr);
            let key = Key::<Aes256Gcm>::try_from(&derived_key[..])
                .map_err(|_| anyhow!("invalid derived key length"))?;
            let cipher = Aes256Gcm::new(&key);

            let nonce = Nonce::try_from(&nonce_arr[..])
                .map_err(|_| anyhow!("invalid nonce length"))?;

            let plaintext = cipher
                .decrypt(&nonce, ciphertext)
                .map_err(|_| anyhow!("Decryption failed for {}: invalid key or data", file))?;

            fs::write(path, plaintext).context("failed to write decrypted file")?;
        }
        Ok(())
    }

    fn normalize_key(key: &str, salt: &[u8]) -> [u8; 32] {
        let mut normalized = [0u8; 32];

        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        );

        argon2
            .hash_password_into(key.as_bytes(), salt, &mut normalized)
            .expect("Argon2 normalization failed");

        normalized
    }
}