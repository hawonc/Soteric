use anyhow::Result;
use clap::Parser;
use std::fs;

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

    let secret_key = fs::read_to_string("secret.txt")?;

    match cli.command {
        Command::AddProfile {
            name,
            file,
            glob,
            activate,
        } => {
            add_profile(&name, file, glob, &mut state)?;
            if activate {
                activate_profile(&name, &mut state)?;
            }
            save_profiles(&profile_file, &state)?;
        }
        Command::AppendProfile { name, file, glob } => {
            append_profile(&name, file, glob, &mut state)?;
            save_profiles(&profile_file, &state)?;
        }
        Command::DeleteProfile { name, yes } => {
            let removed = delete_profile(&name, yes, &mut state)?;
            if removed {
                save_profiles(&profile_file, &state)?;
            }
        }
        Command::Activate { name } => {
            if state.active_profile.is_some() {
                let files = active_profile_files(&state)?;
                Encrypter::decrypt(&files, &secret_key)?;
            }

            activate_profile(&name, &mut state)?;
            save_profiles(&profile_file, &state)?;
        }
        Command::Deactivate { name } => {
            if state.active_profile.is_some() {
                let files = active_profile_files(&state)?;
                Encrypter::decrypt(&files, &secret_key)?;
            }

            deactivate_profile(&name, &mut state)?;
            save_profiles(&profile_file, &state)?;
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
        Command::EncryptNow => {
            let files = active_profile_files(&state)?;
            Encrypter::encrypt(&files, &secret_key)?;
        }
        Command::DecryptNow => {
            let files = active_profile_files(&state)?;
            Encrypter::decrypt(&files, &secret_key)?;
        }
        Command::Run => println!("[TODO] run service not implemented yet"),
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
