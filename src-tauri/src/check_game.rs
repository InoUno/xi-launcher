use std::path::PathBuf;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::fs::{self};

use crate::{
    config::profiles::{AuthKind, Profile},
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum LaunchStatus {
    NeedsGameDir,
    NeedsInstall,
    NeedsAndCanInstall { download_info: DownloadInfo },
    NeedsAshita,
    NeedsUpdate { versions_info: VersionsInfo },
    NeedsPassword,
    Ready,
}

pub async fn check_game_launch(id: u32, state: AppState<'_>) -> anyhow::Result<LaunchStatus> {
    let mut state = state.write().await;

    let profile = state
        .profiles
        .map
        .get(&id)
        .ok_or(anyhow!("No profile with the given index found."))?;

    let Some(game_directory) = &profile.install.directory else {
        return Ok(LaunchStatus::NeedsGameDir);
    };

    if !game_directory.join("FINAL FANTASY XI").exists() {
        tracing::debug!("Missing game directory. Checking for server install URL.");
        if let Some(download_info) = get_server_game_install(profile).await {
            return Ok(LaunchStatus::NeedsAndCanInstall { download_info });
        } else {
            return Ok(LaunchStatus::NeedsInstall);
        }
    }

    let ashita_directory = profile
        .install
        .ashita_directory
        .as_ref()
        .cloned()
        .unwrap_or(game_directory.join("Ashita"));

    if !ashita_directory.exists() {
        return Ok(LaunchStatus::NeedsAshita);
    }

    tracing::debug!("Checking if update is needed.");
    if let Some(versions_info) = needs_update(profile, &ashita_directory).await {
        // Cache versions response
        let server = profile.server.clone().unwrap_or_default();
        state.update_cache.insert(server, versions_info.clone());

        return Ok(LaunchStatus::NeedsUpdate { versions_info });
    }

    tracing::debug!("Checking if password input is required.");
    if !profile.is_retail && !profile.manual_auth {
        match profile.auth_kind {
            AuthKind::Token => {
                if !profile
                    .get_token_path()
                    .map(|path| path.exists())
                    .unwrap_or_default()
                {
                    tracing::debug!("Password needed for token authentication.");
                    return Ok(LaunchStatus::NeedsPassword);
                }
            }
            AuthKind::Password => {
                if profile.password.is_none() {
                    tracing::debug!("Plaintext password is not set.");
                    return Ok(LaunchStatus::NeedsPassword);
                }
            }
            AuthKind::ManualPassword => {
                tracing::debug!("Manual password");
                return Ok(LaunchStatus::NeedsPassword);
            }
        }
    }

    Ok(LaunchStatus::Ready)
}

async fn get_server_game_install(profile: &Profile) -> Option<DownloadInfo> {
    if profile.is_retail {
        return None;
    }

    let Some(_server) = &profile.server else {
        return None;
    };

    let info_addr = profile.get_server_info_addr();

    let response = reqwest::get(format!("http://{info_addr}/install"))
        .await
        .ok()?;

    response.json().await.ok()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VersionsInfo {
    #[serde(default)]
    pub dats: Option<VersionInfo>,

    #[serde(default)]
    pub bootloader: Option<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VersionInfo {
    pub url: String,
    pub version: String,
}

pub type DownloadInfo = Vec<FileInstallConfig>;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct FileInstallConfig {
    pub url: String,
}

async fn needs_update(profile: &Profile, ashita_directory: &PathBuf) -> Option<VersionsInfo> {
    let Some(versions_info) = get_versions_info(profile).await else {
        return None;
    };

    let server_filename = profile.get_server_filename();

    let bootloader_versions_path =
        ashita_directory.join(format!("bootloader/{}/version.txt", server_filename));
    if part_needs_update(&versions_info.bootloader, &bootloader_versions_path)
        .await
        .is_some()
    {
        return Some(versions_info);
    }

    let dats_versions_path =
        ashita_directory.join(format!("polplugins/DATs/{}/version.txt", server_filename));
    if part_needs_update(&versions_info.dats, &dats_versions_path)
        .await
        .is_some()
    {
        return Some(versions_info);
    }

    return None;
}

pub async fn part_needs_update<'a>(
    version_info: &'a Option<VersionInfo>,
    version_path: &PathBuf,
) -> Option<&'a VersionInfo> {
    if let Some(info) = &version_info {
        if !version_path.exists() {
            return Some(&info);
        }

        let Ok(file_version) = fs::read_to_string(version_path).await else {
            return Some(&info);
        };

        if file_version != info.version {
            return Some(&info);
        }
    }

    None
}

pub async fn get_versions_info(profile: &Profile) -> Option<VersionsInfo> {
    if profile.is_retail {
        return None;
    }

    if profile.server.is_none() {
        return None;
    };

    let info_addr = profile.get_server_info_addr();

    let response = reqwest::get(format!("http://{info_addr}/versions"))
        .await
        .inspect_err(|err| {
            tracing::warn!("Could not fetch update information from {info_addr}: {err:?}");
        })
        .ok()?;

    let updates_info: VersionsInfo = response.json().await.ok()?;

    Some(updates_info)
}
