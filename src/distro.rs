use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use iced::task::{Straw, sipper};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
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
    pub fn get_all() -> Result<Vec<Distro>> {
        let iso_metadata =
            reqwest::blocking::get("https://wiki.t2linux.org/tools/distro-metadata.json")?;
        let iso_metadata: DistroMetadataWrapper = serde_json::from_reader(iso_metadata)?;
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
            let mut iso_file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&iso_path)
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
                    iso_file.write_all(&data).with_context(|| {
                        format!("Failed to write to file: {}", iso_path.display())
                    })?;
                    current_len += data.len() as u64;
                    if let Some(total_len) = total_len {
                        sender
                            .send((part, (current_len as f64) / (total_len as f64)))
                            .await;
                    } else {
                        sender.send((part, 0.0)).await;
                    }
                }
            }
            if let Some(compression_algo) = &s.iso_compression {
                match compression_algo {
                    CompressionAlgorithim::Zip => {
                        let mut archive = zip::ZipArchive::new(iso_file.try_clone().unwrap())
                            .with_context(|| {
                                format!("Failed to open ISO to decompress: {}", iso_path.display())
                            })?;
                        let mut decompressed_file = archive.by_index(0)?;
                        std::io::copy(&mut decompressed_file, &mut iso_file).with_context(
                            || format!("Failed to decompress ISO to file: {}", iso_path.display()),
                        )?;
                    }
                }
            }
            fs::OpenOptions::new()
                .read(true)
                .open(&iso_path)
                .with_context(|| format!("Failed to open output ISO file: {}", iso_path.display()))
        })
    }
}
