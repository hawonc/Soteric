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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let original_content = "secret data";
        fs::write(&test_file, original_content).unwrap();

        let file_path = test_file.to_str().unwrap().to_string();
        let key = "my-secure-key";

        // Encrypt
        Encrypter::encrypt(&[file_path.clone()], key).unwrap();
        let encrypted_content = fs::read(&test_file).unwrap();
        assert_ne!(encrypted_content, original_content.as_bytes());

        // Decrypt
        Encrypter::decrypt(&[file_path], key).unwrap();
        let decrypted_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(decrypted_content, original_content);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "secret data").unwrap();

        let file_path = test_file.to_str().unwrap().to_string();

        // Encrypt with one key
        Encrypter::encrypt(&[file_path.clone()], "key1").unwrap();

        // Try to decrypt with different key
        let result = Encrypter::decrypt(&[file_path], "key2");
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("empty.txt");
        fs::write(&test_file, "").unwrap();

        let file_path = test_file.to_str().unwrap().to_string();
        let key = "test-key";

        Encrypter::encrypt(&[file_path.clone()], key).unwrap();
        Encrypter::decrypt(&[file_path], key).unwrap();
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_encrypt_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "data1").unwrap();
        fs::write(&file2, "data2").unwrap();

        let paths = vec![
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];
        let key = "test-key";

        Encrypter::encrypt(&paths, key).unwrap();
        Encrypter::decrypt(&paths, key).unwrap();

        assert_eq!(fs::read_to_string(&file1).unwrap(), "data1");
        assert_eq!(fs::read_to_string(&file2).unwrap(), "data2");
    }

    #[test]
    fn test_normalize_key_consistency() {
        let key = "test-password";
        let salt = [1u8; 16];

        let result1 = Encrypter::normalize_key(key, &salt);
        let result2 = Encrypter::normalize_key(key, &salt);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_normalize_key_different_salts() {
        let key = "test-password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];

        let result1 = Encrypter::normalize_key(key, &salt1);
        let result2 = Encrypter::normalize_key(key, &salt2);

        assert_ne!(result1, result2);
    }

    #[test]
    fn test_decrypt_corrupted_file_fails() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("corrupted.txt");
        fs::write(&test_file, "too short").unwrap();

        let file_path = test_file.to_str().unwrap().to_string();
        let result = Encrypter::decrypt(&[file_path], "any-key");
        assert!(result.is_err());
    }
}