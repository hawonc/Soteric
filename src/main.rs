use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path};
use std::sync::mpsc;
use std::thread;

use soteric::cli::{Cli, Command};
use soteric::models::ProfileState;
use soteric::process_scan::DetectedProcess;
use soteric::process_scan::scan_agent_processes;
use soteric::profiles::{
    activate_profile, deactivate_profile, add_profile, append_profile, current_profile_store_path, delete_profile,
    list_profiles, load_profiles, save_profiles, show_profile, active_profile_files,
};
use soteric::encrypter::Encrypter;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let profile_file = current_profile_store_path()?;
    let mut state: ProfileState = load_profiles(&profile_file)?;

    let mut secret_key = get_secret_key()?;

    match cli.command {
        Command::AddProfile {
            name,
            file,
            glob,
            activate,
        } => {
            add_profile(&name, file, glob, &mut state)?;
            if activate {
                activate_and_encrypt_profile(&name, &mut state, &secret_key, &profile_file)?;
            }
            else {
                save_profiles(&profile_file, &state)?;
            }
        }
        Command::AppendProfile { name, file, glob } => {
            if state.active_profile.as_deref() == Some(&name) {
                println!("Cannot append to an active profile. Please deactivate it first.");
            }
            else {
                append_profile(&name, file, glob, &mut state)?;
                save_profiles(&profile_file, &state)?;
            }
        }
        Command::DeleteProfile { name, yes } => {
            if state.active_profile.as_deref() == Some(&name) {
                println!("Cannot delete an active profile. Please deactivate it first.");
            }
            else {
                let removed = delete_profile(&name, yes, &mut state)?;
                if removed {
                    save_profiles(&profile_file, &state)?;
                }
            }
        }
        Command::Activate { name } => {
            activate_and_encrypt_profile(&name, &mut state, &secret_key, &profile_file)?;
        }
        Command::Deactivate { name } => {
            deactivate_and_decrypt_profile(&name, &mut state, &secret_key, &profile_file)?;
        }
        Command::ShowProfile { name } => show_profile(&name, &state)?,
        Command::ListProfiles => list_profiles(&state),
        Command::Scan => {
            let processes = scan_agent_processes()?;
            print_detected_processes(&processes, true);
        }
        Command::Status => {
            println!("Status");
            match state.active_profile.as_deref() {
                Some(name) => {
                    println!();
                    println!("Active profile:");
                    show_profile(name, &state)?;
                }
                None => {
                    println!();
                    println!("Active profile:");
                    println!("  none");
                }
            }

            println!();
            println!("AI-tool detections:");
            let processes = scan_agent_processes()?;
            if processes.is_empty() {
                println!("  none");
            } else {
                print_detected_processes(&processes, false);
            }
        }
        Command::SetSecret { secret } => {
            println!("Setting new secret.");
            let active_name = state.active_profile.clone()
                .ok_or_else(|| anyhow::anyhow!("No active profile"))?;
            let profile = state.profiles.get_mut(&active_name)
                .ok_or_else(|| anyhow::anyhow!("Active profile not found"))?;
            if profile.encrypted {
                Encrypter::decrypt(&profile.files, &secret_key)?;
            }
            secret_key = secret;
            Encrypter::encrypt(&profile.files, &secret_key)?;
            profile.encrypted = true;
            save_profiles(&profile_file, &state)?;
        }
        Command::SetMapping { process, profile } => {
            if !state.profiles.contains_key(&profile) {
                println!("Profile '{}' does not exist. Cannot set mapping.", profile);
            }
            else {
                state.process_to_profile.insert(process.clone(), profile.clone());
                save_profiles(&profile_file, &state)?;
                println!("Set mapping: process '{}' -> profile '{}'", process, profile);
            }
        },
        Command::DeleteMapping { process } => {
            if state.process_to_profile.remove(&process).is_some() {
                save_profiles(&profile_file, &state)?;
                println!("Deleted mapping for process '{}'", process);
            }
            else {
                println!("No mapping found for process '{}'", process);
            }
        },
        Command::ListMappings => {
            if state.process_to_profile.is_empty() {
                println!("No process-to-profile mappings configured.");
            }
            else {
                println!("Current process-to-profile mappings:");
                for (process, profile) in &state.process_to_profile {
                    println!("  process '{}' -> profile '{}'", process, profile);
                }
            }
        },
        Command::SetupBiometric => {
            #[cfg(target_os = "macos")]
            {
                soteric::biometrics::store_biometric_secret(&secret_key)?;
                println!("Biometric authentication set up successfully.");
                println!("Touch ID will now be used to unlock your encryption key.");
            }
            #[cfg(not(target_os = "macos"))]
            {
                println!("Biometric authentication is only supported on macOS.");
            }
        },
        Command::RemoveBiometric => {
            #[cfg(target_os = "macos")]
            {
                soteric::biometrics::delete_biometric_secret()?;
                println!("Biometric authentication removed.");
            }
            #[cfg(not(target_os = "macos"))]
            {
                println!("Biometric authentication is only supported on macOS.");
            }
        },
        Command::Run => {
            let (tx, rx) = mpsc::channel();

            thread::spawn(move || {
                let mut input = String::new();
                loop {
                    input.clear();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        if input.trim() == "q" {
                            let _ = tx.send(());
                            break;
                        }
                    }
                }
            });
            
            println!("Entering monitoring mode. Press 'q' and Enter to exit.");

            loop {
                std::thread::sleep(std::time::Duration::from_secs(5));
                
                if rx.try_recv().is_ok() {
                    println!("Exiting monitoring mode.");
                    break;
                }

                let processes = scan_agent_processes()?;

                let mut temp_new_process: Option<String> = None;
                for process in &processes {
                    if state.process_to_profile.get(&process.name).is_some() {
                        // found a new process that has a mapping to a profile
                        temp_new_process = Some(process.name.clone());
                        break;
                    }
                }

                if temp_new_process.as_ref() == state.active_process.as_ref() {
                    continue;
                }

                if temp_new_process.is_none() && state.active_process.is_none() {
                    continue;
                }

                if temp_new_process != state.active_process && state.active_process.is_some() {
                    let current_process_name = state.active_process.clone().unwrap();
                    let current_profile_name = state
                        .process_to_profile
                        .get(&current_process_name)
                        .cloned()
                        .unwrap();
                    state.active_process = None;
                    deactivate_and_decrypt_profile(&current_profile_name, &mut state, &secret_key, &profile_file)?;
                    println!(
                        "\t=> for process '{}'",
                        current_process_name
                    );
                }

                if temp_new_process.is_some() {
                    let new_process_name = temp_new_process.clone().unwrap();
                    let new_profile_name = state
                        .process_to_profile
                        .get(&new_process_name)
                        .cloned()
                        .unwrap();
                    state.active_process = Some(new_process_name.clone());
                    activate_and_encrypt_profile(&new_profile_name, &mut state, &secret_key, &profile_file)?;
                    println!(
                        "\t=> for process '{}'",
                        new_process_name
                    );
                }
            }
        },
    }

    Ok(())
}

fn activate_and_encrypt_profile(
    name: &str,
    state: &mut ProfileState,
    secret_key: &str,
    profile_file: &Path) -> Result<()>
{
    // Decrypt old active profile only if its files are encrypted
    if let Some(old_name) = state.active_profile.clone() {
        if let Some(old_profile) = state.profiles.get_mut(&old_name) {
            if old_profile.encrypted {
                Encrypter::decrypt(&old_profile.files, secret_key)?;
                old_profile.encrypted = false;
            }
        }
    }

    activate_profile(name, state)?;

    // Encrypt new profile only if not already encrypted
    let needs_encrypt = !state.profiles[name].encrypted;
    if needs_encrypt {
        state.profiles.get_mut(name).unwrap().encrypted = true;
    }
    save_profiles(profile_file, state)?;
    if needs_encrypt {
        let files = &state.profiles[name].files;
        Encrypter::encrypt(files, secret_key)?;
    }

    Ok(())
}

fn deactivate_and_decrypt_profile(
    name: &str,
    state: &mut ProfileState,
    secret_key: &str,
    profile_file: &Path) -> Result<()>
{
    if state.active_profile.as_deref() == Some(name) {
        // Decrypt only if files are encrypted
        let profile = state.profiles.get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;
        if profile.encrypted {
            Encrypter::decrypt(&profile.files, secret_key)?;
            profile.encrypted = false;
        }

        deactivate_profile(name, state)?;
        save_profiles(profile_file, state)?;
    }

    Ok(())
}

fn print_detected_processes(processes: &[DetectedProcess], include_heading: bool) {
    if processes.is_empty() {
        println!("No agent-orchestrator processes detected.");
        return;
    }

    if include_heading {
        println!("Detected agent-orchestrator processes:");
    }
    for process in processes {
        println!("[{}] {} - {}", process.pid, process.name, process.command);
    }
}

fn get_secret_key() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        match soteric::biometrics::retrieve_biometric_secret() {
            Ok(secret) => return Ok(secret),
            Err(_) => {}
        }
    }
    fs::read_to_string("secret.txt").map(|s| s.trim().to_string())
        .context("Failed to read secret.txt. Set up biometric auth with 'setup-biometric' or create secret.txt.")
}
