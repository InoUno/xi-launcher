use std::{collections::BTreeMap, path::PathBuf};

use super::serde_util;
use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct Profiles {
    #[serde(default)]
    pub ids: Vec<u32>,

    #[serde(default)]
    pub map: BTreeMap<u32, Profile>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct Profile {
    pub id: u32,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub server: Option<String>,

    #[serde(default)]
    pub server_nickname: Option<String>,

    #[serde(default)]
    pub server_info_addr: Option<String>,

    #[serde(skip_serializing_if = "serde_util::is_false")]
    #[serde(default)]
    pub is_retail: bool,

    #[serde(default)]
    pub install: InstallConfig,

    #[serde(default)]
    pub account_name: Option<String>,

    #[serde(default)]
    pub password: Option<String>,

    #[serde(skip_serializing_if = "serde_util::is_default")]
    #[serde(default)]
    pub auth_kind: AuthKind,

    #[serde(skip_serializing_if = "serde_util::is_false")]
    #[serde(default)]
    pub manual_auth: bool,

    #[serde(default)]
    pub hairpin: bool,

    #[serde(default)]
    pub resolution: Resolution,

    #[serde(default)]
    pub background_resolution: Resolution,

    #[serde(default)]
    pub menu_resolution: Resolution,

    #[serde(default)]
    pub enabled_addons: Option<Vec<String>>,

    #[serde(default)]
    pub enabled_plugins: Option<Vec<String>>,

    #[serde(skip_serializing_if = "serde_util::vec_is_empty")]
    #[serde(default)]
    pub extra_pivots: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
#[repr(u8)]
pub enum AuthKind {
    #[default]
    Token = 0,
    Password = 1,
    ManualPassword = 2,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Type)]
pub struct InstallConfig {
    #[serde(default)]
    pub directory: Option<PathBuf>,

    /// If None, and [InstallConfig::directory] is set, then [InstallConfig::directory] is assumed to have Ashita in it as well.
    #[serde(default)]
    pub ashita_directory: Option<PathBuf>,
}

impl InstallConfig {
    pub fn get_ashita_dir(&self) -> Option<PathBuf> {
        self.ashita_directory
            .clone()
            .or(self.directory.as_ref().map(|dir| dir.join("Ashita")))
    }

    pub fn try_get_ashita_dir(&self) -> anyhow::Result<PathBuf> {
        self.ashita_directory
            .clone()
            .or(self.directory.as_ref().map(|dir| dir.join("Ashita")))
            .ok_or_else(|| anyhow::anyhow!("Missing Ashita directory."))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub struct Resolution {
    pub width: u16,
    pub height: u16,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

pub const PROFILES_CONFIG_FILENAME: &'static str = "profiles.json";

impl Profiles {
    pub fn get_path(dir: &PathBuf) -> PathBuf {
        dir.join(PROFILES_CONFIG_FILENAME)
    }

    pub fn add_new_profile(&mut self, mut profile: Profile) {
        let new_id = self.ids.iter().copied().max().unwrap_or(1) + 1;
        self.ids.push(new_id);
        profile.id = new_id;
        self.map.insert(new_id, profile);
    }
}

impl Profile {
    pub fn get_token_path(&self) -> Option<PathBuf> {
        Some(self.install.get_ashita_dir()?.join(format!(
            "bootloader/{}/{}.token",
            self.get_server_filename(),
            self.account_name.clone().unwrap_or_else(|| self.id.to_string())
        )))
    }

    pub fn get_profile_filename(&self) -> String {
        self.name
            .clone()
            .unwrap_or("anon".to_string())
            .replace(" ", "_")
    }

    pub fn get_server_filename(&self) -> String {
        let filename = self.server.as_ref().cloned().unwrap_or_else(|| {
            if self.is_retail {
                "retail"
            } else {
                "localhost"
            }
            .to_string()
        });

        let sanitizer = Regex::new(r"\.\\/").unwrap();
        sanitizer.replace_all(&filename, "_").to_string()
    }

    pub fn get_server_info_addr(&self) -> String {
        self.server_info_addr.clone().unwrap_or_else(|| {
            format!(
                "{}:15850",
                self.server
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| "localhost".to_string())
            )
        })
    }
}
