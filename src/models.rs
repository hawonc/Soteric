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
    #[serde(default)]
    pub encrypted: bool,
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
