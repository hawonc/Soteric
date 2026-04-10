use serde::{Deserialize, Serialize};
use soteric::encrypter::Encrypter;
use soteric::profiles::{
    active_profile_files, load_profiles, save_profiles,
};
use soteric::process_scan::scan_agent_processes;
use std::path::PathBuf;

fn global_profile_path() -> PathBuf {
    if let Some(data) = dirs::data_dir() {
        data.join("soteric").join("profiles.json")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".soteric").join("profiles.json")
    }
}

fn resolve_secret(user_secret: Option<String>) -> Result<String, String> {
    if let Some(s) = user_secret {
        if !s.is_empty() {
            return Ok(s);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(s) = soteric::biometrics::retrieve_biometric_secret() {
            return Ok(s);
        }
    }
    std::fs::read_to_string("secret.txt")
        .map(|s| s.trim().to_string())
        .map_err(|_| "No secret key available. Please provide a password.".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub root: String,
    pub files: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
}

#[tauri::command]
fn list_profiles() -> Result<Vec<ProfileInfo>, String> {
    let path = global_profile_path();
    let state = load_profiles(&path).map_err(|e| e.to_string())?;

    let mut profiles: Vec<ProfileInfo> = state
        .profiles
        .iter()
        .map(|(name, profile)| ProfileInfo {
            name: name.clone(),
            root: profile.root.clone(),
            files: profile.files.clone(),
            active: state.active_profile.as_deref() == Some(name),
        })
        .collect();

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

#[tauri::command]
fn activate_profile(name: String, secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if !state.profiles.contains_key(&name) {
        return Err(format!("Profile '{}' not found", name));
    }

    let key = resolve_secret(secret)?;

    // Decrypt current active profile first
    if state.active_profile.is_some() {
        let files = active_profile_files(&state).map_err(|e| e.to_string())?;
        Encrypter::decrypt(files, &key).map_err(|e| e.to_string())?;
    }

    // Activate and encrypt the new profile
    soteric::profiles::activate_profile(&name, &mut state).map_err(|e| e.to_string())?;
    let files = active_profile_files(&state).map_err(|e| e.to_string())?;
    Encrypter::encrypt(files, &key).map_err(|e| e.to_string())?;

    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn deactivate_profile(name: String, secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if state.active_profile.as_deref() != Some(&name) {
        return Err(format!("Profile '{}' is not active", name));
    }

    let key = resolve_secret(secret)?;

    let files = active_profile_files(&state).map_err(|e| e.to_string())?;
    Encrypter::decrypt(files, &key).map_err(|e| e.to_string())?;

    soteric::profiles::deactivate_profile(&name, &mut state).map_err(|e| e.to_string())?;
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn encrypt_now(secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let state = load_profiles(&path).map_err(|e| e.to_string())?;

    let key = resolve_secret(secret)?;
    let files = active_profile_files(&state).map_err(|e| e.to_string())?;
    Encrypter::encrypt(files, &key).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn decrypt_now(secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let state = load_profiles(&path).map_err(|e| e.to_string())?;

    let key = resolve_secret(secret)?;
    let files = active_profile_files(&state).map_err(|e| e.to_string())?;
    Encrypter::decrypt(files, &key).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_profile(name: String) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if state.active_profile.as_deref() == Some(&name) {
        return Err("Cannot delete an active profile. Deactivate it first.".to_string());
    }

    state.profiles.remove(&name);
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn create_profile(name: String, files: Vec<String>, globs: Vec<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    soteric::profiles::add_profile(&name, file_paths, globs, &mut state)
        .map_err(|e| e.to_string())?;
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn append_profile(name: String, files: Vec<String>, globs: Vec<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if state.active_profile.as_deref() == Some(&name) {
        return Err("Cannot append to an active profile. Deactivate it first.".to_string());
    }

    let file_paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    soteric::profiles::append_profile(&name, file_paths, globs, &mut state)
        .map_err(|e| e.to_string())?;
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn scan_processes() -> Result<Vec<ProcessInfo>, String> {
    let processes = scan_agent_processes().map_err(|e| e.to_string())?;
    Ok(processes
        .into_iter()
        .map(|p| ProcessInfo {
            pid: p.pid,
            name: p.name,
            command: p.command,
        })
        .collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            activate_profile,
            deactivate_profile,
            delete_profile,
            create_profile,
            append_profile,
            encrypt_now,
            decrypt_now,
            scan_processes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
