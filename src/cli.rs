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
    /// Unmark a profile as the active profile.
    Deactivate { name: String },
    /// Show a single profile with its metadata and tracked files.
    #[command(name = "show-profile")]
    ShowProfile { name: String },
    /// Show all configured profiles and their tracked files.
    ListProfiles,
    /// Scan running processes for known AI coding tools.
    Scan,
    /// Show the active profile and current AI-tool detections.
    Status,
    /// Allows the user to set the secret instead of pulling from the file 'secrets.txt'
    SetSecret { secret: String },
    /// Allows the user to set a mapping from a process name to a profile, so that when that process is detected, the corresponding profile is automatically activated.
    SetMapping {
        #[arg(long)]
        process: String,
        #[arg(long)]
        profile: String,
    },
    /// Allows the user to delete an existing process-to-profile mapping.
    DeleteMapping {
        process: String,
    },
    /// List the current process-to-profile mappings.
    ListMappings,
    /// Set up biometric (Touch ID) authentication for the encryption key (macOS only).
    #[command(name = "setup-biometric")]
    SetupBiometric,
    /// Remove biometric authentication (macOS only).
    #[command(name = "remove-biometric")]
    RemoveBiometric,
    /// Start a long-running background process that monitors for AI coding tools and activates profiles accordingly.
    Run,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_add_profile() {
        let args = vec![
            "soteric",
            "add-profile",
            "test",
            "--file",
            "/path/to/file",
        ];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_parsing_activate() {
        let args = vec!["soteric", "activate", "my-profile"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_parsing_scan() {
        let args = vec!["soteric", "scan"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_parsing_status() {
        let args = vec!["soteric", "status"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_parsing_set_mapping() {
        let args = vec![
            "soteric",
            "set-mapping",
            "--process",
            "codex",
            "--profile",
            "secrets",
        ];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }
}
