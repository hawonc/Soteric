use serde::{Deserialize, Serialize};
use soteric::profiles::{load_profiles, save_profiles};
use soteric::process_scan::scan_agent_processes;
use std::path::PathBuf;

fn global_profile_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".soteric").join("profiles.json")
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
fn activate_profile(name: String) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if !state.profiles.contains_key(&name) {
        return Err(format!("Profile '{}' not found", name));
    }

    state.active_profile = Some(name);
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn deactivate_profile(name: String) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if state.active_profile.as_deref() == Some(&name) {
        state.active_profile = None;
        save_profiles(&path, &state).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn delete_profile(name: String) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    state.profiles.remove(&name);
    if state.active_profile.as_deref() == Some(&name) {
        state.active_profile = None;
    }
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
            scan_processes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
