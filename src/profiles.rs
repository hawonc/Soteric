use crate::models::{Profile, ProfileState, ProfileStore, StoredProfile};
use anyhow::{Result, anyhow};
use glob::glob;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub fn current_profile_store_path() -> Result<PathBuf> {
    Ok(current_workspace_root()?.join(crate::models::PROFILE_STORE_FILE))
}

pub fn current_workspace_root() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    Ok(find_workspace_root(&current_dir))
}

pub fn load_profiles(profile_file: &Path) -> Result<ProfileState> {
    if !profile_file.exists() {
        return Ok(ProfileState::empty());
    }

    let mut file = File::open(profile_file)?;
    let mut raw = String::new();
    file.read_to_string(&mut raw)?;
    if raw.trim().is_empty() {
        return Ok(ProfileState::empty());
    }

    let parsed: ProfileStore = serde_json::from_str(&raw)?;
    let profiles: std::collections::HashMap<_, _> = parsed
        .profiles
        .into_iter()
        .map(|(name, entry)| {
            let profile = migrate_profile(name.as_str(), entry);
            (name, profile)
        })
        .collect();

    let active_profile = parsed
        .active_profile
        .filter(|name| profiles.contains_key(name));

    Ok(ProfileState {
        profiles,
        active_profile,
    })
}

pub fn save_profiles(profile_file: &Path, state: &ProfileState) -> Result<()> {
    if let Some(parent) = profile_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let profiles = state
        .profiles
        .iter()
        .map(|(name, profile)| (name.clone(), StoredProfile::Detailed(profile.clone())))
        .collect::<BTreeMap<_, _>>();
    let payload = serde_json::to_string_pretty(&ProfileStore {
        profiles,
        active_profile: state.active_profile.clone(),
    })?;

    let mut file = File::create(profile_file)?;
    file.write_all(payload.as_bytes())?;
    Ok(())
}

pub fn add_profile(
    name: &str,
    files: Vec<PathBuf>,
    globs: Vec<String>,
    state: &mut ProfileState,
) -> Result<()> {
    validate_profile_inputs(&files, &globs, "Profile creation requires")?;

    if state.profiles.contains_key(name) {
        let overwrite = prompt("Profile exists. Overwrite? [y/N]: ")?;
        if !confirmed(&overwrite) {
            println!("Profile unchanged. Exiting.");
            return Ok(());
        }
    }

    let resolved_files = resolve_profile_files(files, globs.clone())?;
    let profile = Profile {
        root: estimate_profile_root(&resolved_files)?,
        created_with: Some(build_creation_info(globs.len(), resolved_files.len())),
        files: resolved_files,
    };

    state.profiles.insert(name.to_string(), profile);
    println!("Added profile '{name}'.");
    Ok(())
}

pub fn append_profile(
    name: &str,
    files: Vec<PathBuf>,
    globs: Vec<String>,
    state: &mut ProfileState,
) -> Result<()> {
    validate_profile_inputs(&files, &globs, "Profile update requires")?;

    let profile = state
        .profiles
        .get_mut(name)
        .ok_or_else(|| anyhow!("Profile '{name}' not found"))?;

    let new_files = resolve_profile_files(files, globs.clone())?;
    profile.files = merge_files(&profile.files, &new_files);
    profile.root = estimate_profile_root(&profile.files)?;

    let existing_globs = profile
        .created_with
        .as_deref()
        .and_then(parse_creation_info)
        .map(|(_, glob_count)| glob_count)
        .unwrap_or_default();
    profile.created_with = Some(build_creation_info(
        existing_globs + globs.len(),
        profile.files.len(),
    ));

    println!("Updated profile '{name}'.");
    Ok(())
}

pub fn delete_profile(name: &str, force: bool, state: &mut ProfileState) -> Result<bool> {
    if !state.profiles.contains_key(name) {
        println!("Profile '{name}' not found.");
        return Ok(false);
    }

    if !force {
        let confirm = prompt(&format!("Delete profile '{name}'? [y/N]: "))?;
        if !confirmed(&confirm) {
            println!("Profile unchanged. Exiting.");
            return Ok(false);
        }
    }

    state.profiles.remove(name);
    if state.active_profile.as_deref() == Some(name) {
        state.active_profile = None;
    }

    println!("Profile '{name}' deleted.");
    Ok(true)
}

pub fn activate_profile(name: &str, state: &mut ProfileState) -> Result<()> {
    if !state.profiles.contains_key(name) {
        return Err(anyhow!("Profile '{name}' not found"));
    }

    state.active_profile = Some(name.to_string());
    println!("Activated profile '{name}'.");
    Ok(())
}

pub fn deactivate_profile(name: &str, state: &mut ProfileState) -> Result<()> {
    if !state.profiles.contains_key(name) {
        return Err(anyhow!("Profile '{name}' not found"));
    }

    state.active_profile = None;
    println!("Deactivated profile '{name}'.");
    Ok(())
}

pub fn show_profile(name: &str, state: &ProfileState) -> Result<()> {
    let profile = state
        .profiles
        .get(name)
        .ok_or_else(|| anyhow!("Profile '{name}' not found"))?;

    print_profile(name, profile, state.active_profile.as_deref() == Some(name));
    Ok(())
}

pub fn list_profiles(state: &ProfileState) {
    if state.profiles.is_empty() {
        println!("No profiles configured.");
        return;
    }

    let mut names = state.profiles.keys().collect::<Vec<_>>();
    names.sort_unstable();

    for name in names {
        let profile = &state.profiles[name];
        print_profile(
            name,
            profile,
            state.active_profile.as_deref() == Some(name.as_str()),
        );
    }
}

fn migrate_profile(name: &str, entry: StoredProfile) -> Profile {
    match entry {
        StoredProfile::Legacy(root) => {
            println!("Migrating legacy profile '{name}'...");
            Profile {
                root,
                files: Vec::new(),
                created_with: Some("legacy-root".into()),
            }
        }
        StoredProfile::LegacyDetailed {
            root,
            blacklisted_files,
            created_with,
        } => Profile {
            root,
            files: blacklisted_files,
            created_with,
        },
        StoredProfile::Detailed(profile) => profile,
    }
}

fn validate_profile_inputs(files: &[PathBuf], globs: &[String], prefix: &str) -> Result<()> {
    if files.is_empty() && globs.is_empty() {
        return Err(anyhow!(
            "{prefix} --file and/or --glob entries. Interactive prompts are disabled."
        ));
    }

    Ok(())
}

fn resolve_profile_files(files: Vec<PathBuf>, globs: Vec<String>) -> Result<Vec<String>> {
    let mut candidates = files;
    for pattern in globs {
        candidates.extend(glob_to_paths(&pattern)?);
    }

    let resolved = candidates
        .into_iter()
        .map(resolve_file_path)
        .collect::<Result<Vec<_>>>()?;
    if resolved.is_empty() {
        return Err(anyhow!("No files matched --file and --glob inputs."));
    }

    Ok(unique_sorted_files(resolved))
}

fn resolve_file_path(path: PathBuf) -> Result<String> {
    let workspace_root = current_workspace_root()?;
    let candidate = if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    };

    let canonical = candidate.canonicalize()?;
    if !canonical.is_file() {
        return Err(anyhow!("{} is not a file", canonical.display()));
    }

    Ok(canonical.to_string_lossy().to_string())
}

fn glob_to_paths(pattern: &str) -> Result<Vec<PathBuf>> {
    let workspace_root = current_workspace_root()?;
    let raw_pattern = if Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        workspace_root.join(pattern).to_string_lossy().to_string()
    };

    let mut matches = Vec::new();
    for entry in glob(&raw_pattern)? {
        match entry {
            Ok(path) if path.is_file() => matches.push(path),
            Ok(_) => {}
            Err(err) => return Err(anyhow!("glob pattern error: {err}")),
        }
    }

    Ok(matches)
}

fn estimate_profile_root(files: &[String]) -> Result<String> {
    let first = files
        .first()
        .ok_or_else(|| anyhow!("Cannot estimate a profile root without files"))?;
    let first = Path::new(first);
    if !first.is_file() {
        return Err(anyhow!(
            "Could not determine root from {} because it is not a file",
            first.display()
        ));
    }

    let first_parent = first.parent().unwrap_or(Path::new("."));
    let same_parent = files
        .iter()
        .skip(1)
        .all(|path| Path::new(path).parent() == Some(first_parent));

    if same_parent {
        return Ok(first_parent.to_string_lossy().to_string());
    }

    Ok(current_workspace_root()?.to_string_lossy().to_string())
}

fn merge_files(existing: &[String], new_files: &[String]) -> Vec<String> {
    unique_sorted_files(
        existing
            .iter()
            .chain(new_files.iter())
            .cloned()
            .collect::<Vec<_>>(),
    )
}

fn build_creation_info(glob_count: usize, file_count: usize) -> String {
    let mut parts = Vec::new();
    if file_count > 0 {
        parts.push(format!("file:{file_count}"));
    }
    if glob_count > 0 {
        parts.push(format!("glob:{glob_count}"));
    }
    parts.join(", ")
}

fn parse_creation_info(value: &str) -> Option<(usize, usize)> {
    let mut file_count = 0usize;
    let mut glob_count = 0usize;

    for part in value.split(',').map(str::trim) {
        if let Some(count) = part.strip_prefix("file:") {
            file_count = count.parse().ok()?;
        } else if let Some(count) = part.strip_prefix("glob:") {
            glob_count = count.parse().ok()?;
        }
    }

    Some((file_count, glob_count))
}

fn unique_sorted_files(files: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for file in files {
        seen.insert(file);
    }
    seen.into_iter().collect()
}

fn confirmed(response: &str) -> bool {
    response.trim().eq_ignore_ascii_case("y")
}

fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}

fn print_profile(name: &str, profile: &Profile, is_active: bool) {
    let marker = if is_active { "*" } else { " " };
    let display_name = format!("\x1b[1m{name}\x1b[0m");

    println!("{marker} {display_name}");
    println!("    root: {}", profile.root);
    if let Some(created_with) = &profile.created_with {
        println!("    created_with: {created_with}");
    }
    println!("    files ({}):", profile.files.len());
    for file in &profile.files {
        println!("    - {file}");
    }
}

fn find_workspace_root(start: &Path) -> PathBuf {
    for candidate in start.ancestors() {
        if candidate.join(".git").exists() {
            return candidate.to_path_buf();
        }
    }

    start.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::{
        ProfileState, add_profile, append_profile, build_creation_info, estimate_profile_root,
        find_workspace_root, load_profiles, parse_creation_info, resolve_profile_files,
        save_profiles,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn save_and_load_profiles_round_trip() {
        let temp_dir = create_temp_dir("round-trip");
        let profile_path = temp_dir.join("profiles.json");
        let file_path = temp_dir.join("one.txt");
        fs::write(&file_path, "secret").unwrap();

        let mut state = ProfileState::empty();
        add_profile("demo", vec![file_path.clone()], Vec::new(), &mut state).unwrap();
        state.active_profile = Some("demo".into());
        save_profiles(&profile_path, &state).unwrap();

        let loaded = load_profiles(&profile_path).unwrap();
        assert_eq!(loaded.active_profile.as_deref(), Some("demo"));
        assert_eq!(loaded.profiles["demo"].files.len(), 1);

        cleanup_dir(&temp_dir);
    }

    #[test]
    fn resolve_profile_files_supports_absolute_globs_and_dedupes() {
        let temp_dir = create_temp_dir("glob");
        let first = temp_dir.join("one.txt");
        let second = temp_dir.join("two.txt");
        fs::write(&first, "one").unwrap();
        fs::write(&second, "two").unwrap();

        let pattern = format!("{}/{}.txt", temp_dir.display(), "*");
        let resolved = resolve_profile_files(vec![first.clone()], vec![pattern]).unwrap();

        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|path| path.ends_with("one.txt")));
        assert!(resolved.iter().any(|path| path.ends_with("two.txt")));

        cleanup_dir(&temp_dir);
    }

    #[test]
    fn append_profile_merges_new_files() {
        let temp_dir = create_temp_dir("append");
        let first = temp_dir.join("one.txt");
        let second = temp_dir.join("two.txt");
        fs::write(&first, "one").unwrap();
        fs::write(&second, "two").unwrap();

        let mut state = ProfileState::empty();
        add_profile("demo", vec![first], Vec::new(), &mut state).unwrap();
        append_profile("demo", vec![second], Vec::new(), &mut state).unwrap();

        let profile = &state.profiles["demo"];
        assert_eq!(profile.files.len(), 2);
        assert_eq!(profile.created_with.as_deref(), Some("file:2"));

        cleanup_dir(&temp_dir);
    }

    #[test]
    fn root_is_parent_for_single_directory_profiles() {
        let temp_dir = create_temp_dir("root");
        let first = temp_dir.join("one.txt");
        let second = temp_dir.join("two.txt");
        fs::write(&first, "one").unwrap();
        fs::write(&second, "two").unwrap();

        let root = estimate_profile_root(&[
            first.canonicalize().unwrap().to_string_lossy().to_string(),
            second.canonicalize().unwrap().to_string_lossy().to_string(),
        ])
        .unwrap();

        assert_eq!(Path::new(&root), temp_dir.canonicalize().unwrap().as_path());

        cleanup_dir(&temp_dir);
    }

    #[test]
    fn creation_info_round_trips() {
        let value = build_creation_info(2, 5);
        assert_eq!(value, "file:5, glob:2");
        assert_eq!(parse_creation_info(&value), Some((5, 2)));
    }

    #[test]
    fn finds_repo_root_from_nested_directory() {
        let temp_dir = create_temp_dir("repo-root");
        let repo_root = temp_dir.join("repo");
        let nested = repo_root.join("src").join("nested");
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        fs::create_dir_all(&nested).unwrap();

        assert_eq!(find_workspace_root(&nested), repo_root);

        cleanup_dir(&temp_dir);
    }

    fn create_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("soteric-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn cleanup_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
