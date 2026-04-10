use serde::{Deserialize, Serialize};
use soteric::encrypter::Encrypter;
use soteric::profiles::{
    active_profile_files, load_profiles, save_profiles,
};
use soteric::process_scan::scan_agent_processes;
use chrono::Timelike;
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;

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
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            encrypted: profile.encrypted,
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

    // Decrypt old active profile only if encrypted
    if let Some(old_name) = state.active_profile.clone() {
        if let Some(old_profile) = state.profiles.get_mut(&old_name) {
            if old_profile.encrypted {
                Encrypter::decrypt(&old_profile.files, &key).map_err(|e| e.to_string())?;
                old_profile.encrypted = false;
            }
        }
    }

    // Activate new profile
    soteric::profiles::activate_profile(&name, &mut state).map_err(|e| e.to_string())?;

    // Encrypt new profile only if not already encrypted
    let needs_encrypt = !state.profiles[&name].encrypted;
    if needs_encrypt {
        state.profiles.get_mut(&name).unwrap().encrypted = true;
    }
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    if needs_encrypt {
        Encrypter::encrypt(&state.profiles[&name].files, &key).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn deactivate_profile(name: String, secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if state.active_profile.as_deref() != Some(&name) {
        return Err(format!("Profile '{}' is not active", name));
    }

    // Decrypt only if encrypted — no secret needed if already decrypted
    let profile = state.profiles.get_mut(&name)
        .ok_or_else(|| format!("Profile '{}' not found", name))?;
    if profile.encrypted {
        let key = resolve_secret(secret)?;
        Encrypter::decrypt(&profile.files, &key).map_err(|e| e.to_string())?;
        profile.encrypted = false;
    }

    soteric::profiles::deactivate_profile(&name, &mut state).map_err(|e| e.to_string())?;
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn encrypt_now(secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    let active_name = state.active_profile.clone()
        .ok_or("No active profile")?;
    let profile = state.profiles.get_mut(&active_name)
        .ok_or("Active profile not found")?;

    if profile.encrypted {
        return Ok(()); // Already encrypted, no-op
    }

    let key = resolve_secret(secret)?;
    profile.encrypted = true;
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Encrypter::encrypt(&state.profiles[&active_name].files, &key).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn decrypt_now(secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    let active_name = state.active_profile.clone()
        .ok_or("No active profile")?;
    let profile = state.profiles.get_mut(&active_name)
        .ok_or("Active profile not found")?;

    if !profile.encrypted {
        return Ok(()); // Already decrypted, no-op
    }

    let key = resolve_secret(secret)?;
    profile.encrypted = false;
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Encrypter::decrypt(&state.profiles[&active_name].files, &key).map_err(|e| e.to_string())?;
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

// --- Secret management ---

#[tauri::command]
fn set_secret(current_secret: Option<String>, new_secret: String) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    // If there's an active profile with encrypted files, re-encrypt with the new key
    if let Some(active_name) = state.active_profile.clone() {
        if let Some(profile) = state.profiles.get_mut(&active_name) {
            if profile.encrypted {
                let old_key = resolve_secret(current_secret)?;
                Encrypter::decrypt(&profile.files, &old_key).map_err(|e| e.to_string())?;
                Encrypter::encrypt(&profile.files, &new_secret).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn setup_biometric(secret: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        soteric::biometrics::store_biometric_secret(&secret).map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = secret;
        Err("Biometric authentication is only supported on macOS.".to_string())
    }
}

#[tauri::command]
fn remove_biometric() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        soteric::biometrics::delete_biometric_secret().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Biometric authentication is only supported on macOS.".to_string())
    }
}

#[tauri::command]
fn check_biometric() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(soteric::biometrics::retrieve_biometric_secret().is_ok())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(false)
    }
}

// --- Process-to-profile mappings ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingInfo {
    pub process: String,
    pub profile: String,
}

#[tauri::command]
fn list_mappings() -> Result<Vec<MappingInfo>, String> {
    let path = global_profile_path();
    let state = load_profiles(&path).map_err(|e| e.to_string())?;

    let mut mappings: Vec<MappingInfo> = state
        .process_to_profile
        .iter()
        .map(|(process, profile)| MappingInfo {
            process: process.clone(),
            profile: profile.clone(),
        })
        .collect();

    mappings.sort_by(|a, b| a.process.cmp(&b.process));
    Ok(mappings)
}

#[tauri::command]
fn set_mapping(process: String, profile: String) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    if !state.profiles.contains_key(&profile) {
        return Err(format!("Profile '{}' does not exist", profile));
    }

    state.process_to_profile.insert(process, profile);
    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_mapping(process: String, secret: Option<String>) -> Result<(), String> {
    let path = global_profile_path();
    let mut state = load_profiles(&path).map_err(|e| e.to_string())?;

    let profile_name = state.process_to_profile.remove(&process)
        .ok_or_else(|| format!("No mapping found for process '{}'", process))?;

    // If this mapping's process was the active_process, deactivate + decrypt
    if state.active_process.as_deref() == Some(&process) {
        if let Some(profile) = state.profiles.get_mut(&profile_name) {
            if profile.encrypted {
                let key = resolve_secret(secret)?;
                Encrypter::decrypt(&profile.files, &key).map_err(|e| e.to_string())?;
                profile.encrypted = false;
            }
        }
        if state.active_profile.as_deref() == Some(&profile_name) {
            soteric::profiles::deactivate_profile(&profile_name, &mut state)
                .map_err(|e| e.to_string())?;
        }
        state.active_process = None;
    }

    save_profiles(&path, &state).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Background monitor ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitorScanEvent {
    processes: Vec<ProcessInfo>,
    time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitorActivityEvent {
    event: String,
    time: String,
}

fn now_time() -> String {
    let local = chrono::Local::now();
    format!("{}:{:02}", local.hour(), local.minute())
}

struct MonitorState {
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[tauri::command]
fn start_monitor(secret: Option<String>, monitor: State<MonitorState>, app: AppHandle) -> Result<(), String> {
    use std::sync::atomic::Ordering::Relaxed;

    if monitor.running.load(Relaxed) {
        return Err("Monitor is already running".to_string());
    }
    monitor.running.store(true, Relaxed);
    monitor.stop.store(false, Relaxed);

    let key = resolve_secret(secret)?;
    let stop = monitor.stop.clone();
    let running = monitor.running.clone();

    // Track processes that the user manually decrypted — don't re-encrypt
    // until the process disappears and comes back (new session).
    std::thread::spawn(move || {
        let mut user_dismissed: HashSet<String> = HashSet::new();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));

            if stop.load(Relaxed) {
                running.store(false, Relaxed);
                break;
            }

            let path = global_profile_path();
            let mut state = match load_profiles(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let processes = match scan_agent_processes() {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Emit scan results to frontend
            let process_infos: Vec<ProcessInfo> = processes
                .iter()
                .map(|p| ProcessInfo {
                    pid: p.pid,
                    name: p.name.clone(),
                    command: p.command.clone(),
                })
                .collect();

            let time = now_time();
            let _ = app.emit("monitor-scan", MonitorScanEvent {
                processes: process_infos,
                time: time.clone(),
            });

            // Clear stale active_process if its mapping no longer exists
            if let Some(ref ap) = state.active_process {
                if !state.process_to_profile.contains_key(ap) {
                    state.active_process = None;
                    let _ = save_profiles(&path, &state);
                }
            }

            // Find first detected process that has a mapping
            // Check both p.name AND keywords in p.command against mapping keys
            let mut new_process: Option<String> = None;
            for p in &processes {
                if state.process_to_profile.contains_key(&p.name) {
                    new_process = Some(p.name.clone());
                    break;
                }
                let cmd_lower = p.command.to_lowercase();
                for mapped_proc in state.process_to_profile.keys() {
                    let mapped_lower = mapped_proc.to_lowercase();
                    if cmd_lower.split_whitespace().any(|token| {
                        let basename = token.rsplit('/').next().unwrap_or(token);
                        let basename = basename.rsplit('\\').next().unwrap_or(basename);
                        basename.strip_suffix(".exe").unwrap_or(basename) == mapped_lower
                    }) {
                        new_process = Some(mapped_proc.clone());
                        break;
                    }
                }
                if new_process.is_some() { break; }
            }

            // Clean up dismissed set: if a process disappeared, remove it
            // so next time it appears it will trigger again (new session).
            let detected_names: HashSet<String> = processes.iter()
                .map(|p| p.name.clone())
                .collect();
            user_dismissed.retain(|name| {
                // Keep in dismissed only if the process is still running
                detected_names.contains(name) || processes.iter().any(|p| {
                    p.command.to_lowercase().contains(&name.to_lowercase())
                })
            });

            // Skip if user manually decrypted this process's profile
            if let Some(ref np) = new_process {
                if user_dismissed.contains(np) {
                    // Process still running but user dismissed — don't re-encrypt
                    continue;
                }
            }

            if new_process == state.active_process {
                // Check if user manually decrypted the active profile
                if let Some(ap) = state.active_process.clone() {
                    if let Some(profile_name) = state.process_to_profile.get(&ap).cloned() {
                        if let Some(profile) = state.profiles.get(&profile_name) {
                            if !profile.encrypted {
                                // User manually decrypted — remember this and stop tracking
                                user_dismissed.insert(ap.clone());
                                state.active_process = None;
                                if state.active_profile.as_deref() == Some(&profile_name) {
                                    let _ = soteric::profiles::deactivate_profile(&profile_name, &mut state);
                                }
                                let _ = save_profiles(&path, &state);
                                let _ = app.emit("monitor-activity", MonitorActivityEvent {
                                    event: format!("User decrypted '{}' manually — monitor will not re-encrypt until '{}' restarts", profile_name, ap),
                                    time: time.clone(),
                                });
                            }
                        }
                    }
                }
                continue;
            }

            // --- State change: deactivate old, activate new ---

            // Deactivate current if needed
            if let Some(current) = state.active_process.clone() {
                if let Some(profile_name) = state.process_to_profile.get(&current).cloned() {
                    let result = (|| -> Result<(), anyhow::Error> {
                        if let Some(profile) = state.profiles.get_mut(&profile_name) {
                            if profile.encrypted {
                                Encrypter::decrypt(&profile.files, &key)?;
                                profile.encrypted = false;
                            }
                        }
                        soteric::profiles::deactivate_profile(&profile_name, &mut state)?;
                        save_profiles(&path, &state)?;
                        Ok(())
                    })();
                    let msg = match result {
                        Ok(()) => format!("Auto-deactivated profile '{}' (process '{}' stopped)", profile_name, current),
                        Err(e) => format!("Error auto-deactivating '{}': {}", profile_name, e),
                    };
                    let _ = app.emit("monitor-activity", MonitorActivityEvent {
                        event: msg.clone(),
                        time: time.clone(),
                    });
                    // Send OS notification
                    let _ = app.notification()
                        .builder()
                        .title("Soteric")
                        .body(&msg)
                        .show();
                }
            }
            state.active_process = None;

            // Activate new if needed
            if let Some(ref new_proc) = new_process {
                if let Some(profile_name) = state.process_to_profile.get(new_proc).cloned() {
                    let result = (|| -> Result<(), anyhow::Error> {
                        soteric::profiles::activate_profile(&profile_name, &mut state)?;
                        let needs_encrypt = !state.profiles[&profile_name].encrypted;
                        if needs_encrypt {
                            state.profiles.get_mut(&profile_name).unwrap().encrypted = true;
                        }
                        state.active_process = new_process.clone();
                        save_profiles(&path, &state)?;
                        if needs_encrypt {
                            Encrypter::encrypt(&state.profiles[&profile_name].files, &key)?;
                        }
                        Ok(())
                    })();
                    let msg = match result {
                        Ok(()) => format!("Auto-activated profile '{}' — files encrypted (detected '{}')", profile_name, new_proc),
                        Err(e) => format!("Error auto-activating '{}': {}", profile_name, e),
                    };
                    let _ = app.emit("monitor-activity", MonitorActivityEvent {
                        event: msg.clone(),
                        time: time.clone(),
                    });
                    // Send OS notification
                    let _ = app.notification()
                        .builder()
                        .title("Soteric")
                        .body(&msg)
                        .show();
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_monitor(monitor: State<MonitorState>) -> Result<(), String> {
    use std::sync::atomic::Ordering::Relaxed;
    if !monitor.running.load(Relaxed) {
        return Err("Monitor is not running".to_string());
    }
    monitor.stop.store(true, Relaxed);
    Ok(())
}

#[tauri::command]
fn is_monitor_running(monitor: State<MonitorState>) -> Result<bool, String> {
    Ok(monitor.running.load(std::sync::atomic::Ordering::Relaxed))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(MonitorState {
            running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            stop: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
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
            set_secret,
            setup_biometric,
            remove_biometric,
            check_biometric,
            list_mappings,
            set_mapping,
            delete_mapping,
            start_monitor,
            stop_monitor,
            is_monitor_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
