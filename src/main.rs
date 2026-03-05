use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use glob::glob;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "soteric", version, about = "Very basic Soteric skeleton")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    AddProfile {
        name: String,
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        browse: bool,
        #[arg(long)]
        pick_blacklist: bool,
        #[arg(long)]
        blacklist: Vec<PathBuf>,
        #[arg(long)]
        blacklist_from: Option<PathBuf>,
        #[arg(long)]
        blacklist_glob: Vec<String>,
    },
    RemoveProfile {
        name: String,
        #[arg(short, long)]
        yes: bool,
    },
    ListProfiles,
    EncryptNow,
    DecryptNow,
    Run,
}

const PROFILE_STORE_FILE: &str = ".soteric/profiles.json";

type Profiles = HashMap<String, Profile>;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let profile_file = current_profile_store_path()?;
    let mut profiles = load_profiles(&profile_file)?;

    match cli.command {
        Command::AddProfile {
            name,
            path,
            browse,
            pick_blacklist,
            blacklist,
            blacklist_from,
            blacklist_glob,
        } => {
            add_profile(
                &name,
                path,
                browse,
                pick_blacklist,
                blacklist,
                blacklist_from,
                blacklist_glob,
                &mut profiles,
            )?;
            save_profiles(&profile_file, &profiles)?;
        }
        Command::RemoveProfile { name, yes } => {
            let removed = remove_profile(&name, yes, &mut profiles)?;
            if removed {
                save_profiles(&profile_file, &profiles)?;
            }
        }
        Command::ListProfiles => list_profiles(&profiles),
        Command::EncryptNow => println!("[TODO] encrypt-now not implemented yet"),
        Command::DecryptNow => println!("[TODO] decrypt-now not implemented yet"),
        Command::Run => println!("[TODO] run service not implemented yet"),
    }

    Ok(())
}

fn remove_profile(name: &str, force: bool, profiles: &mut Profiles) -> Result<bool> {
    if !profiles.contains_key(name) {
        println!("Profile '{name}' not found.");
        return Ok(false);
    }

    if !force {
        let confirm = prompt(&format!("Delete profile '{name}'? [y/N]: "))?;
        if confirm.trim().to_lowercase() != "y" {
            println!("Profile unchanged. Exiting.");
            return Ok(false);
        }
    }

    profiles.remove(name);
    println!("Profile '{name}' deleted.");
    Ok(true)
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredProfiles {
    profiles: HashMap<String, StoredProfile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StoredProfile {
    Legacy(String),
    Detailed(Profile),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Profile {
    root: String,
    blacklisted_files: Vec<String>,
    created_with: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BlacklistFileData {
    List(Vec<String>),
    Object { blacklist: Vec<String> },
    Single(String),
}

impl Profile {
    fn with_root(root: PathBuf) -> Self {
        Self {
            root: root.to_string_lossy().to_string(),
            blacklisted_files: Vec::new(),
            created_with: None,
        }
    }

    fn with_blacklist(
        mut self,
        blacklisted_files: Vec<String>,
        created_with: Option<String>,
    ) -> Self {
        self.blacklisted_files = blacklisted_files;
        self.created_with = created_with;
        self
    }
}

fn add_profile(
    name: &str,
    path: Option<PathBuf>,
    browse: bool,
    pick_blacklist: bool,
    blacklist: Vec<PathBuf>,
    blacklist_from: Option<PathBuf>,
    blacklist_glob: Vec<String>,
    profiles: &mut Profiles,
) -> Result<()> {
    let target_dir = if browse {
        pick_directory()?
    } else {
        match path {
            Some(path) => resolve_directory(path)?,
            None => resolve_directory_from_prompt()?,
        }
    };

    if profiles.contains_key(name) {
        let overwrite = prompt("Profile exists. Overwrite? [y/N]: ")?;
        if overwrite.trim().to_lowercase() != "y" {
            println!("Profile unchanged. Exiting.");
            return Ok(());
        }
    }

    let file_count = count_files_in_directory(&target_dir)?;
    let created_with = build_creation_info(
        blacklist_from.as_ref(),
        &blacklist,
        pick_blacklist,
        &blacklist_glob,
    );
    let blacklisted_files = resolve_file_list(
        &target_dir,
        pick_blacklist,
        blacklist,
        blacklist_from,
        blacklist_glob,
    )?;

    profiles.insert(
        name.to_string(),
        Profile::with_root(target_dir.clone()).with_blacklist(blacklisted_files, created_with),
    );

    println!(
        "Added profile '{name}' -> {} ({} files, {} blacklisted)",
        target_dir.display(),
        file_count,
        profiles
            .get(name)
            .map(|p| p.blacklisted_files.len())
            .unwrap_or(0),
    );

    Ok(())
}

fn list_profiles(profiles: &Profiles) {
    if profiles.is_empty() {
        println!("No profiles configured.");
        return;
    }

    println!("Configured profiles:");
    for (name, profile) in profiles {
        println!("{name}: {}", profile.root);
        let creator = profile
            .created_with
            .clone()
            .unwrap_or_else(|| String::from("legacy/manual"));
        println!("  source: {creator}");
        match profile.blacklisted_files.len() {
            0 => println!("  blacklisted files: 0"),
            1 => {
                println!("  blacklisted file:");
                println!("    - {}", profile.blacklisted_files[0]);
            }
            n => {
                println!("  blacklisted files ({n}):");
                for file in &profile.blacklisted_files {
                    println!("    - {file}");
                }
            }
        }
    }
}

fn load_profiles(profile_file: &Path) -> Result<Profiles> {
    if !profile_file.exists() {
        return Ok(HashMap::new());
    }

    let mut file = File::open(profile_file)?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    if raw.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let parsed: StoredProfiles = serde_json::from_str(&raw)?;
    let mut migrated = HashMap::new();
    for (name, entry) in parsed.profiles {
        let profile = match entry {
            StoredProfile::Legacy(root) => Profile {
                root,
                blacklisted_files: Vec::new(),
                created_with: Some("legacy-root".into()),
            },
            StoredProfile::Detailed(profile) => profile,
        };
        migrated.insert(name, profile);
    }

    Ok(migrated)
}

fn save_profiles(profile_file: &Path, profiles: &Profiles) -> Result<()> {
    if let Some(parent) = profile_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = serde_json::to_string_pretty(&StoredProfiles {
        profiles: profiles
            .iter()
            .map(|(name, profile)| (name.to_owned(), StoredProfile::Detailed(profile.clone())))
            .collect(),
    })?;
    let mut file = File::create(profile_file)?;
    file.write_all(payload.as_bytes())?;
    Ok(())
}

fn current_profile_store_path() -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    Ok(cwd.join(PROFILE_STORE_FILE))
}

fn resolve_directory(path: PathBuf) -> Result<PathBuf> {
    let candidate = if path.is_absolute() {
        path
    } else {
        env::current_dir()?.join(path)
    };

    let canonical = candidate.canonicalize()?;
    if !canonical.is_dir() {
        return Err(anyhow!("{} is not a directory", canonical.display()));
    }

    Ok(canonical)
}

fn resolve_file_path(base_dir: &Path, path: PathBuf) -> Result<String> {
    let candidate = if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    };

    let canonical = candidate.canonicalize()?;
    if !canonical.is_file() {
        return Err(anyhow!("{} is not a file", canonical.display()));
    }

    Ok(canonical.to_string_lossy().to_string())
}

fn resolve_directory_from_prompt() -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    let prompt_text = format!(
        "No directory provided. Press Enter to use current directory [{}]: ",
        cwd.display()
    );
    let input = prompt(&prompt_text)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(cwd.canonicalize()?);
    }

    resolve_directory(PathBuf::from(trimmed))
}

fn resolve_file_list(
    base_dir: &Path,
    pick_blacklist: bool,
    explicit: Vec<PathBuf>,
    from_file: Option<PathBuf>,
    globs: Vec<String>,
) -> Result<Vec<String>> {
    let mut files = explicit;

    if let Some(from_file) = from_file {
        files.extend(load_blacklist_file(base_dir, &from_file)?);
    }

    if pick_blacklist {
        let dialog = FileDialog::new()
            .set_title("Select blacklisted files")
            .set_directory(base_dir);
        if let Some(selected) = dialog.pick_files() {
            files.extend(selected);
        }
    }

    for pattern in globs {
        files.extend(glob_to_paths(base_dir, &pattern)?);
    }

    let resolved = files
        .into_iter()
        .map(|path| resolve_file_path(base_dir, path))
        .collect::<Result<Vec<_>>>()?;

    let unique_sorted = {
        let mut set = BTreeSet::new();
        for item in resolved {
            set.insert(item);
        }
        set.into_iter().collect::<Vec<_>>()
    };

    Ok(unique_sorted)
}

fn load_blacklist_file(base_dir: &Path, source: &Path) -> Result<Vec<PathBuf>> {
    let resolved = resolve_file_path(base_dir, source.to_path_buf())?;
    let raw = fs::read_to_string(&resolved)?;
    let ext = Path::new(&resolved)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let entries: Vec<String> = match ext.as_str() {
        "json" => serde_json::from_str::<BlacklistFileData>(&raw)?.into_list(),
        "toml" => toml::from_str::<BlacklistFileData>(&raw)?.into_list(),
        "yaml" | "yml" => serde_yaml::from_str::<BlacklistFileData>(&raw)?.into_list(),
        _ => parse_plain_blacklist(&raw),
    };

    Ok(entries.into_iter().map(PathBuf::from).collect::<Vec<_>>())
}

fn glob_to_paths(base_dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let raw_pattern = if Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        base_dir.join(pattern).to_string_lossy().to_string()
    };

    let mut matches = Vec::new();
    let iterator = glob(&raw_pattern)?;
    for entry in iterator {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    matches.push(path);
                }
            }
            Err(err) => return Err(anyhow!("glob pattern error: {err}")),
        }
    }

    Ok(matches)
}

fn parse_plain_blacklist(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToString::to_string)
        .collect()
}

fn build_creation_info(
    blacklist_from: Option<&PathBuf>,
    blacklist_paths: &[PathBuf],
    pick_blacklist: bool,
    blacklist_glob: &[String],
) -> Option<String> {
    let mut parts = Vec::<String>::new();

    if let Some(source) = blacklist_from {
        parts.push(format!("from-file: {}", source.display()));
    }

    if !blacklist_paths.is_empty() {
        parts.push(format!("explicit:{}", blacklist_paths.len()));
    }

    if pick_blacklist {
        parts.push("picker".to_string());
    }

    if !blacklist_glob.is_empty() {
        parts.push(format!("glob:{}", blacklist_glob.len()));
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(", "))
}

fn pick_directory() -> Result<PathBuf> {
    if let Some(path) = FileDialog::new()
        .set_title("Select a profile directory")
        .pick_folder()
    {
        return resolve_directory(path);
    }

    println!("No directory selected. Falling back to terminal input.");
    resolve_directory_from_prompt()
}

fn count_files_in_directory(dir: &Path) -> Result<usize> {
    let mut total = 0usize;
    if !dir.is_dir() {
        return Err(anyhow!("{} is not a directory", dir.display()));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            total += count_files_in_directory(&path)?;
            continue;
        }
        total = total.saturating_add(1);
    }

    Ok(total)
}

fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}

impl BlacklistFileData {
    fn into_list(self) -> Vec<String> {
        match self {
            Self::List(values) => values,
            Self::Object { blacklist } => blacklist,
            Self::Single(value) => vec![value],
        }
    }
}
