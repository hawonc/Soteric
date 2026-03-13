use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct Encrypter;

impl Encrypter {
    pub fn encrypt(files: &[String], key: &str) -> Result<()> {
        for file in files {
            Self::process_file(file, key)?;
        }
        Ok(())
    }

    pub fn decrypt(files: &[String], key: &str) -> Result<()> {
        for file in files {
            Self::process_file(file, key)?;
        }
        Ok(())
    }

    fn process_file(file: &str, key: &str) -> Result<()> {
        let path = Path::new(file);

        let data = fs::read(path)
            .with_context(|| format!("failed to read file: {}", path.display()))?;

        let transformed = Self::xor_transform(&data, key.as_bytes());

        fs::write(path, transformed)
            .with_context(|| format!("failed to write file: {}", path.display()))?;

        Ok(())
    }

    fn xor_transform(data: &[u8], key: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, byte)| byte ^ key[i % key.len()])
            .collect()
    }
}