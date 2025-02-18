use std::{fs, path::PathBuf};

use anyhow::anyhow;
use futures_util::StreamExt;
use reqwest::Response;
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    check_game::{part_needs_update, VersionsInfo},
    config::profiles::Profile,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "event", content = "data")]
pub enum UpdateTaskMessage {
    DownloadPending {
        id: &'static str,
    },

    DownloadStarted {
        id: &'static str,
        content_length: u64,
    },

    DownloadProgress {
        id: &'static str,
        finished_length: u64,
    },

    DownloadFinished {
        id: &'static str,
    },

    UnpackPending {
        id: &'static str,
    },

    UnpackFinished {
        id: &'static str,
    },

    FailedSpecific {
        id: &'static str,
    },

    Done,
}

pub async fn update_with_versions(
    profile: &Profile,
    versions_info: VersionsInfo,
    channel: Channel<UpdateTaskMessage>,
) -> anyhow::Result<()> {
    let ashita_directory = profile
        .install
        .get_ashita_dir()
        .ok_or_else(|| anyhow!("Missing Ashita directory."))?;

    let server_filename = profile.get_server_filename();

    let bootloader_base_path = ashita_directory.join(format!("bootloader/{}", server_filename));
    let bootloader_versions_path = bootloader_base_path.join("version.txt");
    if let Some(info) =
        part_needs_update(&versions_info.bootloader, &bootloader_versions_path).await
    {
        tracing::info!(
            "Found bootloader update for {} with version {}",
            server_filename,
            info.version
        );
        fs::create_dir_all(&bootloader_base_path)?;
        let bootloader_path = bootloader_base_path.join("xiloader.exe");

        let id = "Bootloader";
        channel.send(UpdateTaskMessage::DownloadPending { id })?;

        tracing::info!(
            "Downloading bootloader for {} with version {}",
            server_filename,
            info.version
        );
        download_file_to(id, &bootloader_path, &info.url, &channel).await?;

        tracing::info!(
            "Writing bootloader version file for {} with version {}",
            server_filename,
            info.version
        );
        let mut versions_file = File::create(bootloader_versions_path).await?;
        versions_file.write_all(info.version.as_bytes()).await?;
        versions_file.flush().await?;
    }

    let dats_base_path = ashita_directory.join(format!("polplugins/DATs/{}", server_filename));
    let dats_versions_path = dats_base_path.join("version.txt");
    if let Some(info) = part_needs_update(&versions_info.dats, &dats_versions_path).await {
        tracing::info!(
            "Found DATs update for {} with version {}",
            server_filename,
            info.version
        );

        // Clean up previous DATs
        let _ = fs::remove_dir_all(&dats_base_path);

        // Download archive containing new DATs
        fs::create_dir_all(&dats_base_path)?;
        let dats_archive_path = dats_base_path.join("dats.7z");

        tracing::info!(
            "Downloading DATs for {} with version {}",
            server_filename,
            info.version
        );
        let id = "DATs";
        channel.send(UpdateTaskMessage::DownloadPending { id })?;
        download_file_to(id, &dats_archive_path, &info.url, &channel).await?;

        tracing::info!("Unpacking DATs");
        channel.send(UpdateTaskMessage::UnpackPending { id })?;
        sevenz_rust::decompress_file(&dats_archive_path, dats_base_path)?;
        let _ = fs::remove_file(dats_archive_path);
        channel.send(UpdateTaskMessage::UnpackFinished { id })?;

        tracing::info!(
            "Writing version DATs file for {} with version {}",
            server_filename,
            info.version
        );
        let mut versions_file = File::create(dats_versions_path).await?;
        versions_file.write_all(info.version.as_bytes()).await?;
        versions_file.flush().await?;
    }

    tracing::info!("Update complete");
    channel.send(UpdateTaskMessage::Done)?;
    Ok(())
}

async fn download_file_to(
    id: &'static str,
    output_path: &PathBuf,
    url: &str,
    channel: &Channel<UpdateTaskMessage>,
) -> anyhow::Result<()> {
    let file = File::create(output_path).await?;

    let response = reqwest::get(url).await?;

    channel.send(UpdateTaskMessage::DownloadStarted {
        id,
        content_length: response.content_length().unwrap_or_default(),
    })?;

    if let Err(err) = stream_to_file(id, response, file, &channel).await {
        tracing::error!("Error while streaming to file: {err:?}");
        channel.send(UpdateTaskMessage::FailedSpecific { id })?;
    } else {
        channel.send(UpdateTaskMessage::DownloadFinished { id })?;
    }

    Ok(())
}

async fn stream_to_file(
    id: &'static str,
    response: Response,
    mut file: File,
    channel: &Channel<UpdateTaskMessage>,
) -> anyhow::Result<()> {
    let mut stream = response.bytes_stream();

    let mut finished_length = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        finished_length += chunk.len() as u64;

        channel.send(UpdateTaskMessage::DownloadProgress {
            id,
            finished_length,
        })?;
    }

    Ok(())
}
