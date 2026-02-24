use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use iced::task::{Straw, sipper};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Distro {
    pub name: String,
    iso_compression: Option<CompressionAlgorithim>,
    pub iso: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
enum CompressionAlgorithim {
    Zip,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct DistroMetadataWrapper {
    all: Vec<Distro>,
}

impl Distro {
    pub async fn get_all() -> Result<Vec<Distro>> {
        let req = reqwest::get("https://wiki.t2linux.org/tools/distro-metadata.json").await?;
        let iso_metadata = req.bytes().await?;
        let iso_metadata: DistroMetadataWrapper = serde_json::from_slice(&iso_metadata)?;
        Ok(iso_metadata.all)
    }

    pub fn download_iso(
        &self,
        iso_path: PathBuf,
        ct: CancellationToken,
    ) -> impl Straw<fs::File, (usize, f64), anyhow::Error> {
        let s = self.clone();
        sipper(async move |mut sender| {
            let client = reqwest::Client::new();
            fs::remove_file(&iso_path).ok();
            let mut iso_file = tokio::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&iso_path)
                .await
                .with_context(|| format!("Could not open ISO file: {}", &iso_path.display()))?;
            for (part, url) in s.iso.iter().enumerate() {
                let request = client
                    .get(url)
                    .send()
                    .await
                    .with_context(|| format!("Failed to request ISO url: {url}"))?;
                let total_len = request.content_length();
                let mut current_len: u64 = 0;
                let mut data = request.bytes_stream();
                while let Some(Ok(data)) = data.next().await {
                    if ct.is_cancelled() {
                        return Err(anyhow!("Download cancelled"));
                    };
                    iso_file.write_all(&data).await.with_context(|| {
                        format!("Failed to write to file: {}", iso_path.display())
                    })?;
                    current_len += data.len() as u64;
                    if let Some(total_len) = total_len {
                        sender
                            .send((part + 1, (current_len as f64) / (total_len as f64)))
                            .await;
                    } else {
                        sender.send((part + 1, 0.0)).await;
                    }
                }
            }
            if let Some(compression_algo) = &s.iso_compression {
                let decompressed_path = {
                    let mut decompressed_name = iso_path
                        .components()
                        .next_back()
                        .with_context(|| {
                            format!(
                                "Couldn't parse ISO filename from path: {}",
                                iso_path.display()
                            )
                        })?
                        .as_os_str()
                        .to_owned();
                    decompressed_name.push(".extract-tmp");
                    let mut decompressed_path = iso_path
                        .parent()
                        .with_context(|| {
                            format!("Couldn't parse ISO dir from path: {}", iso_path.display())
                        })?
                        .to_owned();
                    decompressed_path.push(&decompressed_name);
                    decompressed_path
                };
                match compression_algo {
                    CompressionAlgorithim::Zip => {
                        let iso_path = iso_path.clone();
                        let decompressed_path = decompressed_path.clone();
                        tokio::task::spawn_blocking(move || -> Result<()> {
                            let mut decompressed_file = fs::OpenOptions::new()
                                .create_new(true)
                                .write(true)
                                .open(&decompressed_path)
                                .with_context(|| {
                                    format!(
                                        "Failed to open temp file for decompressing: {}",
                                        decompressed_path.display()
                                    )
                                })?;
                            let downloaded_file = fs::OpenOptions::new()
                                .read(true)
                                .open(&iso_path)
                                .with_context(|| {
                                    format!(
                                        "Failed to open download to decompress: {}",
                                        iso_path.display()
                                    )
                                })?;
                            let mut archive =
                                zip::ZipArchive::new(downloaded_file.try_clone().unwrap())
                                    .with_context(|| {
                                        format!("Failed to open zip: {}", iso_path.display())
                                    })?;
                            let mut zip_handle = archive.by_index(0)?;
                            std::io::copy(&mut zip_handle, &mut decompressed_file).with_context(
                                || {
                                    format!(
                                        "Failed to decompress ISO to file: {}",
                                        iso_path.display()
                                    )
                                },
                            )?;
                            Ok(())
                        })
                        .await
                        .unwrap()?;
                    }
                }
                let _iso_path = iso_path.clone();
                let _decompressed_path = decompressed_path.clone();
                tokio::task::spawn_blocking(move || -> Result<()> {
                    std::fs::rename(_decompressed_path, &_iso_path).with_context(|| {
                        format!("Failed to move decompressed ISO to {}", _iso_path.display())
                    })?;
                    Ok(())
                })
                .await
                .unwrap()?;
            }
            fs::OpenOptions::new()
                .read(true)
                .open(&iso_path)
                .with_context(|| format!("Failed to open output ISO file: {}", iso_path.display()))
        })
    }
}
