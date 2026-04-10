use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub const PROFILE_STORE_FILE: &str = ".soteric/profiles.json";

pub type Profiles = HashMap<String, Profile>;

#[derive(Debug, Clone)]
pub struct ProfileState {
    pub profiles: Profiles,
    pub active_profile: Option<String>,
    pub process_to_profile: HashMap<String, String>,
    pub active_process : Option<String>,
}

impl ProfileState {
    pub fn empty() -> Self {
        Self {
            profiles: HashMap::new(),
            active_profile: None,
            process_to_profile : HashMap::new(),
            active_process : None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub root: String,
    pub files: Vec<String>,
    pub created_with: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StoredProfile {
    Legacy(String),
    Detailed(Profile),
    LegacyDetailed {
        root: String,
        blacklisted_files: Vec<String>,
        created_with: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: BTreeMap<String, StoredProfile>,
    #[serde(default)]
    pub active_profile: Option<String>,
    #[serde(default)]
    pub process_to_profile: BTreeMap<String, String>,
    #[serde(default)]
    pub active_process: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_state_empty() {
        let state = ProfileState::empty();
        assert!(state.profiles.is_empty());
        assert_eq!(state.active_profile, None);
        assert!(state.process_to_profile.is_empty());
        assert_eq!(state.active_process, None);
    }

    #[test]
    fn test_profile_creation() {
        let profile = Profile {
            root: "/home/user".to_string(),
            files: vec!["/home/user/secret.txt".to_string()],
            created_with: Some("explicit files".to_string()),
        };
        assert_eq!(profile.root, "/home/user");
        assert_eq!(profile.files.len(), 1);
    }

    #[test]
    fn test_profile_state_with_data() {
        let mut state = ProfileState::empty();
        let mut profile = Profile {
            root: "/home/user".to_string(),
            files: vec![],
            created_with: None,
        };
        profile.files.push("/home/user/secret.txt".to_string());

        state.profiles.insert("test-profile".to_string(), profile);
        state.active_profile = Some("test-profile".to_string());

        assert!(state.profiles.contains_key("test-profile"));
        assert_eq!(state.active_profile, Some("test-profile".to_string()));
    }

    #[test]
    fn test_profile_state_process_mapping() {
        let mut state = ProfileState::empty();
        state
            .process_to_profile
            .insert("codex".to_string(), "secrets".to_string());
        state
            .process_to_profile
            .insert("claude".to_string(), "secrets".to_string());

        assert_eq!(state.process_to_profile.len(), 2);
        assert_eq!(
            state.process_to_profile.get("codex"),
            Some(&"secrets".to_string())
        );
    }

    #[test]
    fn test_stored_profile_serialization() {
        let profile = Profile {
            root: "/test".to_string(),
            files: vec!["file1.txt".to_string()],
            created_with: Some("globs".to_string()),
        };

        let stored = StoredProfile::Detailed(profile);
        let json = serde_json::to_string(&stored).unwrap();
        let deserialized: StoredProfile = serde_json::from_str(&json).unwrap();

        match deserialized {
            StoredProfile::Detailed(p) => {
                assert_eq!(p.root, "/test");
                assert_eq!(p.files.len(), 1);
            }
            _ => panic!("Expected Detailed variant"),
        }
    }
}
