use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "soteric",
    version,
    about = "Profile-based file protection for AI coding tools"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or replace a profile from explicit files and/or globs.
    AddProfile {
        name: String,
        /// Add a specific file to the profile. May be passed multiple times.
        #[arg(short, long)]
        file: Vec<PathBuf>,
        /// Expand matching files into the profile. May be passed multiple times.
        #[arg(long)]
        glob: Vec<String>,
        /// Activate this profile after creating or replacing it.
        #[arg(long)]
        activate: bool,
    },
    /// Append files and/or globs to an existing profile.
    #[command(name = "append-profile", alias = "update-profile")]
    AppendProfile {
        name: String,
        /// Add a specific file to the profile. May be passed multiple times.
        #[arg(short, long)]
        file: Vec<PathBuf>,
        /// Expand matching files into the profile. May be passed multiple times.
        #[arg(long)]
        glob: Vec<String>,
    },
    /// Delete a stored profile.
    #[command(name = "delete-profile", aliases = ["remove-profile"])]
    DeleteProfile {
        name: String,
        /// Skip the interactive confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
    /// Mark a profile as the active profile.
    Activate { name: String },
    /// Show a single profile with its metadata and tracked files.
    #[command(name = "show-profile")]
    ShowProfile { name: String },
    /// Show all configured profiles and their tracked files.
    ListProfiles,
    /// Scan running processes for known AI coding tools.
    Scan,
    /// Show the active profile and current AI-tool detections.
    Status,
    /// Placeholder for one-shot encryption of the active profile.
    EncryptNow,
    /// Placeholder for one-shot decryption of the active profile.
    DecryptNow,
    /// Placeholder for a long-running watcher service.
    Run,
}
