use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{ipc::Channel, AppHandle};

use crate::{
    ashita,
    check_game::{check_game_launch, get_versions_info, DownloadInfo, LaunchStatus},
    config::profiles::{AuthKind, Profile, Profiles},
    state::AppState,
    tasks::{
        install::{install_client, InstallTaskProgress},
        update::{update_with_versions, UpdateTaskMessage},
    },
    windower,
};

#[tauri::command]
#[specta::specta]
pub async fn get_profiles(state: AppState<'_>) -> Result<Profiles, String> {
    Ok(state.read().await.profiles.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn save_profile(
    id: Option<u32>,
    profile: Profile,
    state: AppState<'_>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut state = state.write().await;

    if !profile.use_windower {
        ashita::update_ashita_files(&profile, &app_handle)
            .await
            .map_err(|err| format!("Could not update Ashita files: {err:?}"))?;
    }

    if let Some(existing_id) = id {
        tracing::info!("Saving profile {}.", existing_id);
        let Some(existing) = state.profiles.map.get_mut(&existing_id) else {
            return Err(format!("No profile found with ID {}", existing_id));
        };

        *existing = profile;
    } else {
        tracing::info!("Creating new profile");
        state.profiles.add_new_profile(profile);
    };

    state
        .save_configs()
        .map_err(|err| format!("Couldn't save profile to disk: {err:?}"))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn duplicate_profile(id: u32, state: AppState<'_>) -> Result<(), String> {
    let mut state = state.write().await;

    tracing::info!("Duplicating profile at {}", id);

    let Some(profile) = state.profiles.map.get(&id).cloned() else {
        return Err(format!("No profile found with ID {}", id));
    };

    tracing::info!("Duplicating profile {:#?}", profile);
    state.profiles.add_new_profile(profile);

    state
        .save_configs()
        .map_err(|err| format!("Couldn't save configs to disk: {err:?}"))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn move_profile(
    from_index: usize,
    to_index: usize,
    state: AppState<'_>,
) -> Result<(), String> {
    let mut state = state.write().await;

    let profile_len = state.profiles.ids.len();
    if profile_len <= from_index || profile_len <= to_index {
        return Err(format!(
            "Only {profile_len} profiles. Can't move from {from_index} to {to_index}"
        ));
    }

    if from_index == to_index {
        return Ok(());
    }

    tracing::info!("Moving profile from {from_index} to {to_index}");

    if from_index > to_index {
        state.profiles.ids[from_index..to_index].rotate_right(1);
    } else {
        state.profiles.ids[from_index..to_index].rotate_left(1);
    }

    state
        .save_configs()
        .map_err(|err| format!("Couldn't save configs to disk: {err:?}"))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_profile(id: u32, state: AppState<'_>) -> Result<(), String> {
    let mut state = state.write().await;

    if let Some(profile) = state.profiles.map.remove(&id) {
        state
            .profiles
            .ids
            .iter()
            .position(|inner_id| inner_id == &id)
            .map(|pos| {
                state.profiles.ids.remove(pos);
            });

        tracing::info!("Deleted profile {:#?}", profile);

        state
            .save_configs()
            .map_err(|err| format!("Couldn't save configs to disk: {err:?}"))?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum StartProfileResult {
    Started,
    NeedsPassword { account_name: Option<String> },
}

#[tauri::command]
#[specta::specta]
pub async fn should_request_password(id: u32, state: AppState<'_>) -> Result<bool, String> {
    let read_state = state.read().await;

    let profile = read_state
        .profiles
        .map
        .get(&id)
        .ok_or("No profile with the given ID found.".to_string())?;

    if profile.manual_auth {
        return Ok(false);
    }

    match profile.auth_kind {
        AuthKind::Token => {
            return Ok(!profile
                .get_token_path()
                .map(|path| path.exists())
                .unwrap_or_default());
        }
        AuthKind::ManualPassword => {
            return Ok(true);
        }
        AuthKind::Password => {
            return Ok(profile.password.is_none());
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn check_launch_profile(id: u32, state: AppState<'_>) -> Result<LaunchStatus, String> {
    let status = check_game_launch(id, state)
        .await
        .map_err(|err| format!("Failed to check launch of game: {err:?}"));

    status
}

#[tauri::command]
#[specta::specta]
pub async fn update_profile_server_files(
    id: u32,
    state: AppState<'_>,
    channel: Channel<UpdateTaskMessage>,
) -> Result<(), String> {
    let mut state = state.write().await;

    let profile = state
        .profiles
        .map
        .get(&id)
        .ok_or("No profile with the given ID found.".to_string())?;

    let Some(server) = profile.server.clone() else {
        return Err("Expected server to have a name.".to_string());
    };

    let mut versions_info = state.update_cache.remove(&server);

    // Re-get profile after having updated cache
    let profile = state
        .profiles
        .map
        .get(&id)
        .ok_or("No profile with the given ID found.".to_string())?;

    if versions_info.is_none() {
        versions_info = get_versions_info(profile).await;
    }

    match versions_info {
        Some(info) => {
            update_with_versions(profile, info, channel)
                .await
                .map_err(|err| format!("Failed to update game: {err:?}"))?;
        }
        None => {}
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn launch_profile(
    id: u32,
    password: Option<String>,
    state: AppState<'_>,
) -> Result<(), String> {
    let read_state = state.read().await;

    let profile = read_state
        .profiles
        .map
        .get(&id)
        .ok_or("No profile with the given ID found.".to_string())?;

    if profile.use_windower {
        windower::launch_game(profile, password)
            .await
            .map_err(|err| format!("Failed to launch game: {err:?}"))?;
    } else {
        ashita::launch_game(profile, password)
            .await
            .map_err(|err| format!("Failed to launch game: {err:?}"))?;
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn install_game_for_profile(
    id: u32,
    download_info: DownloadInfo,
    state: AppState<'_>,
    channel: Channel<InstallTaskProgress>,
) -> Result<(), String> {
    install_client(id, download_info, state, channel)
        .await
        .map_err(|err| format!("{err:?}"))
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_possible_profile_task(id: u32, state: AppState<'_>) -> Result<(), String> {
    if let Some(token) = state.write().await.ongoing_tasks.remove(&id) {
        token.cancel();
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn list_ashita_addons(ashita_directory: PathBuf) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(ashita_directory.join("addons"))
        .map_err(|err| format!("Couldn't find Ashita addons: {err:?}"))?;

    let mut addons = vec![];
    for entry in entries {
        entry
            .ok()
            .filter(|e| e.file_type().map(|e| e.is_dir()).unwrap_or_default())
            .and_then(|e| e.file_name().into_string().ok())
            .map(|name| addons.push(name));
    }

    Ok(addons)
}

#[tauri::command]
#[specta::specta]
pub async fn list_ashita_plugins(ashita_directory: PathBuf) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(ashita_directory.join("plugins"))
        .map_err(|err| format!("Couldn't find Ashita plugins: {err:?}"))?;

    let mut plugins = vec![];
    for entry in entries {
        entry
            .ok()
            .filter(|e| e.file_type().map(|e| e.is_file()).unwrap_or_default())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "dll")
                    .unwrap_or_default()
            })
            .and_then(|e| {
                e.path()
                    .file_stem()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s.to_owned())
            })
            .map(|name| plugins.push(name.to_owned()));
    }

    Ok(plugins)
}
