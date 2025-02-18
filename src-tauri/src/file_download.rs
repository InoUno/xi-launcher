use std::path::PathBuf;

use anyhow::anyhow;
use futures_util::StreamExt;
use reqwest::Response;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

use crate::task_manager::TaskManager;

#[derive(Debug)]
pub enum DownloadMessage {
    GatheringData,
    Started { content_length: u64 },
    Progress { progress_length: u64 },
    FileDone { path: PathBuf },
    Done,
    Error { message: String },
}

#[derive(Debug)]
pub enum FileDownloadMessage {
    Chunk { chunk_length: u64 },
    Error { path: PathBuf, message: String },
    Done { path: PathBuf },
}

pub async fn download_files(
    urls: impl IntoIterator<Item = String>,
    output_dir: PathBuf,
    tx: mpsc::Sender<DownloadMessage>,
    token: CancellationToken,
) -> anyhow::Result<()> {
    fs::create_dir_all(&output_dir).await?;

    tx.send(DownloadMessage::GatheringData).await?;

    let mut full_content_length = 0u64;
    let mut responses = vec![];

    for url in urls.into_iter() {
        if let Some(filename) = get_filename_from_url(&url) {
            let output_path = output_dir.join(filename);

            if output_path.exists() {
                // File already exists
                tx.send(DownloadMessage::FileDone { path: output_path })
                    .await?;
                continue;
            }
        }

        let response = reqwest::get(&url).await?;
        full_content_length += response.content_length().unwrap_or_default();

        let filename = get_filename_from_response_or_url(&response, &url);
        let output_path = output_dir.join(filename);

        responses.push((output_path, response));
    }

    tx.send(DownloadMessage::Started {
        content_length: full_content_length,
    })
    .await?;

    let mut tm = TaskManager::default();

    let (progress_tx, mut progress_rx) = mpsc::channel(32);
    for ((output_path, response), progress_tx) in
        responses.into_iter().zip(std::iter::repeat(progress_tx))
    {
        tm.tracker.spawn(stream_response_to_file(
            response,
            output_path,
            progress_tx,
            tm.token.clone(),
        ));
    }

    tm.tracker.close();

    let mut progress_length = 0u64;

    loop {
        tokio::select! {
            () = token.cancelled() => {
                tm.shutdown().await;
                return Err(anyhow!("Download cancelled."));
            },
            msg = progress_rx.recv() => {
                if let Some(msg) = msg {
                    match msg {
                        FileDownloadMessage::Chunk { chunk_length } => {
                            progress_length += chunk_length;
                            tx.send(DownloadMessage::Progress { progress_length }).await?;
                        },
                        FileDownloadMessage::Error { path, message } => {
                            fs::remove_file(path).await?;
                            tx.send(DownloadMessage::Error { message }).await?;
                            tm.shutdown().await;
                            return Err(anyhow!("Download failed."));
                        },
                        FileDownloadMessage::Done { path } => {
                            tx.send(DownloadMessage::FileDone { path }).await?;
                        }
                    }
                } else {
                    // All senders are done
                    break;
                }
            }
        }
    }

    tm.tracker.wait().await;

    tx.send(DownloadMessage::Done).await?;

    Ok(())
}

pub async fn stream_response_to_file(
    response: Response,
    output_path: PathBuf,
    tx: mpsc::Sender<FileDownloadMessage>,
    token: CancellationToken,
) {
    let outer_tx = tx.clone();
    if let Err(err) = stream_response_to_file_result(response, &output_path, tx, token).await {
        let _ = outer_tx
            .send(FileDownloadMessage::Error {
                message: format!(
                    "Error while downloading to file '{}': {err}",
                    output_path.display()
                ),
                path: output_path,
            })
            .await;
    }
}

pub async fn stream_response_to_file_result(
    response: Response,
    output_path: &PathBuf,
    tx: mpsc::Sender<FileDownloadMessage>,
    token: CancellationToken,
) -> anyhow::Result<()> {
    let mut file = File::create(output_path).await?;

    let mut stream = response.bytes_stream();

    loop {
        tokio::select! {
            () = token.cancelled() => {
                fs::remove_file(output_path).await?;
                return Ok(());
            }
            chunk = stream.next() => {
                if let Some(chunk) = chunk {
                    let chunk = chunk?;
                    file.write_all(&chunk).await?;

                    tx.send(FileDownloadMessage::Chunk {
                        chunk_length: chunk.len() as u64,
                    })
                    .await?;
                } else {
                    break;
                }
            }
        }
    }

    tx.send(FileDownloadMessage::Done {
        path: output_path.clone(),
    })
    .await?;

    Ok(())
}

pub fn get_filename_from_response_or_url(response: &Response, url: &str) -> String {
    // Try to get the filename from the Content-Disposition header
    let filename = response
        .headers()
        .get("Content-Disposition")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|header_str| {
            header_str.split(';').find_map(|part| {
                let trimmed_part = part.trim();
                if trimmed_part.starts_with("filename=") {
                    // Remove "filename=" and any surrounding quotes
                    let filename_with_quotes = trimmed_part.trim_start_matches("filename=");
                    let filename_without_quotes = filename_with_quotes
                        .trim_start_matches('"')
                        .trim_end_matches('"');
                    Some(filename_without_quotes.to_string())
                } else {
                    None
                }
            })
        });

    match filename {
        Some(name) => name,
        None => {
            // If Content-Disposition header is not present or doesn't contain filename,
            // try to extract it from the URL
            get_filename_from_url(url).unwrap_or("unknown".to_string())
        }
    }
}

fn get_filename_from_url(url: &str) -> Option<String> {
    url.split('/').last().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tokio::{fs, sync::mpsc};
    use tokio_util::sync::CancellationToken;

    use crate::file_download::{download_files, DownloadMessage};

    #[tokio::test]
    #[ignore]
    pub async fn test_download() {
        let (tx, mut rx) = mpsc::channel(32);
        let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/temp");

        tokio::spawn(download_files(
            [
                "https://sh.rustup.rs".to_string(),
                "https://get.pnpm.io/install.ps1".to_string(),
            ],
            out.clone(),
            tx,
            CancellationToken::new(),
        ));

        while let Some(msg) = rx.recv().await {
            match msg {
                DownloadMessage::GatheringData => {
                    eprintln!("Gathering data")
                }
                DownloadMessage::Started { content_length } => {
                    eprintln!("Download started of {content_length} bytes")
                }
                DownloadMessage::Progress { progress_length } => {
                    eprintln!("Download progress {progress_length} bytes")
                }
                DownloadMessage::FileDone { path } => {
                    eprintln!("File completed: {}", path.display());
                }
                DownloadMessage::Done => {
                    eprintln!("Done");
                    break;
                }
                DownloadMessage::Error { message } => {
                    eprintln!("Error: {message}");
                    break;
                }
            }
        }

        let _ = fs::remove_dir_all(out).await;
    }
}
