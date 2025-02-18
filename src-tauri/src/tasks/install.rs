use std::{
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
    u64,
};

use anyhow::{anyhow, Context};
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    check_game::DownloadInfo,
    file_download::{download_files, DownloadMessage},
    state::AppState,
};

pub struct InstallTask;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "event", content = "data")]
pub enum InstallTaskProgress {
    Pending,
    DownloadStarted { content_length: u64 },
    DownloadProgress { finished_length: u64 },
    Installing,
    Complete,
    Error(String),
}

pub async fn install_client(
    id: u32,
    download_info: DownloadInfo,
    state: AppState<'_>,
    channel: Channel<InstallTaskProgress>,
) -> anyhow::Result<()> {
    let read_state = state.read().await;

    let profile = read_state
        .profiles
        .map
        .get(&id)
        .context("No profile with the given ID found.")?;

    let install_dir = profile
        .install
        .directory
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow!("Expected game directory."))?;
    drop(read_state);

    let download_dir = install_dir.join("downloads");

    let token = CancellationToken::new();
    state.write().await.ongoing_tasks.insert(id, token.clone());

    let mut receiver = InstallTask::start(download_info, download_dir, install_dir, token.clone());

    // Return true if it break out of receive loop
    let handle_message = async move |message: Option<InstallTaskProgress>| -> anyhow::Result<bool> {
        let Some(message) = message else {
            channel.send(InstallTaskProgress::Error("Unknown error.".to_string()))?;
            return Ok(true);
        };

        match message {
            msg @ (InstallTaskProgress::Complete | InstallTaskProgress::Error(_)) => {
                channel.send(msg)?;
                return Ok(true);
            }
            msg => {
                channel.send(msg)?;
            }
        }
        Ok(false)
    };

    loop {
        tokio::select! {
            () = token.cancelled() => {
                break;
            }
            msg = receiver.recv() => {
                if let Ok(true) = handle_message(msg).await {
                    break;
                }
            }
        }
    }

    state.write().await.ongoing_tasks.remove(&id);

    Ok(())
}

impl InstallTask {
    pub fn start(
        download_info: DownloadInfo,
        download_dir: PathBuf,
        output_dir: PathBuf,
        token: CancellationToken,
    ) -> mpsc::Receiver<InstallTaskProgress> {
        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            if let Err(err) =
                Self::execute(download_info, download_dir, output_dir, tx.clone(), token).await
            {
                tracing::error!("Error during install: {err:?}");
                let _ = tx.send(InstallTaskProgress::Error(err.to_string())).await;
            }
        });
        rx
    }

    async fn execute(
        download_info: DownloadInfo,
        download_dir: PathBuf,
        output_dir: PathBuf,
        sender: mpsc::Sender<InstallTaskProgress>,
        token: CancellationToken,
    ) -> anyhow::Result<()> {
        if output_dir.join("FINAL FANTASY XI").exists() {
            sender.send(InstallTaskProgress::Complete).await?;
            return Ok(());
        }

        Self::download_step(download_info, download_dir, output_dir, sender, token).await
    }

    async fn download_step(
        download_info: DownloadInfo,
        download_dir: PathBuf,
        output_dir: PathBuf,
        sender: mpsc::Sender<InstallTaskProgress>,
        token: CancellationToken,
    ) -> anyhow::Result<()> {
        if !download_info.iter().any(|file| file.url.ends_with(".exe")) {
            return Err(anyhow!(
                "Expected to find an executable installer in the download file URLs, but it was not present."
            ));
        }

        sender.send(InstallTaskProgress::Pending).await?;

        let (tx, mut rx) = mpsc::channel(32);
        tokio::task::spawn(download_files(
            download_info.into_iter().map(|file| file.url),
            download_dir,
            tx,
            token.clone(),
        ));

        let mut installer_path: Option<PathBuf> = None;

        let mut next_progress_update = Instant::now();

        while let Some(msg) = rx.recv().await {
            match msg {
                DownloadMessage::GatheringData => {}
                DownloadMessage::Started { content_length } => {
                    sender
                        .send(InstallTaskProgress::DownloadStarted { content_length })
                        .await?;
                }
                DownloadMessage::Progress { progress_length } => {
                    let now = Instant::now();
                    if now >= next_progress_update {
                        sender
                            .send(InstallTaskProgress::DownloadProgress {
                                finished_length: progress_length,
                            })
                            .await?;
                        next_progress_update =
                            now.checked_add(Duration::from_secs(1)).unwrap_or(now);
                    }
                }
                DownloadMessage::FileDone { path } => {
                    tracing::info!("Finished downloading file '{}'", path.display());
                    if path.extension().map(|ext| ext == "exe").unwrap_or_default() {
                        installer_path = Some(path);
                    }
                }
                DownloadMessage::Done => {
                    tracing::info!("Done downloading");
                    break;
                }
                DownloadMessage::Error { message } => {
                    sender
                        .send(InstallTaskProgress::Error(message.clone()))
                        .await?;
                    return Err(anyhow!("Failed downloading game client: {message}"));
                }
            }
        }

        if token.is_cancelled() {
            return Ok(());
        }

        let Some(installer_path) = installer_path else {
            return Err(anyhow!("Expected to find an executable installer in the download files, but it was not present."));
        };

        tokio::task::spawn_blocking(move || Self::install_step(output_dir, sender, &installer_path))
            .await?
    }

    fn install_step(
        output_dir: PathBuf,
        sender: mpsc::Sender<InstallTaskProgress>,
        installer_path: &PathBuf,
    ) -> anyhow::Result<()> {
        sender.blocking_send(InstallTaskProgress::Installing)?;

        tracing::info!("Installing using {}", installer_path.to_string_lossy());
        let mut child = Command::new(installer_path)
            .args([
                format!("-o{}", output_dir.to_string_lossy()),
                "-y".to_string(),
            ])
            .spawn()?;

        let out = child.wait()?;

        if !out.success() {
            sender.blocking_send(InstallTaskProgress::Error(format!(
                "Failed to install game using: {}",
                installer_path.to_string_lossy()
            )))?;
            return Ok(());
        }

        if !output_dir.join("FINAL FANTASY XI").exists() {
            sender.blocking_send(InstallTaskProgress::Error(
                "Installed game did not result in a proper game setup.".to_string(),
            ))?;
            return Ok(());
        }

        sender.blocking_send(InstallTaskProgress::Complete)?;
        Ok(())
    }
}
