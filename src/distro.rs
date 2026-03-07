use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use iced::task::{Straw, sipper};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Distro {
    pub name: String,
    iso_compression: Option<CompressionAlgorithim>,
    pub iso: Vec<String>,
    sha256: Option<String>,
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
        file: Arc<File>,
        ct: CancellationToken,
    ) -> impl Straw<(), (usize, f64), anyhow::Error> {
        let s = self.clone();
        let mut hasher = Sha256::new();
        sipper(async move |mut sender| {
            if s.iso_compression.is_some() {
                return Err(anyhow!("Compression is unimplemented"));
            };
            let client = reqwest::Client::new();
            let mut iso_file_buf = BufWriter::new(file.try_clone().await?);
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
                    iso_file_buf
                        .write_all(&data)
                        .await
                        .context("Failed to write to iso download location")?;
                    hasher.update(&data);
                    current_len += data.len() as u64;
                    if let Some(total_len) = total_len {
                        sender
                            .send((part + 1, (current_len as f64) / (total_len as f64)))
                            .await;
                    } else {
                        sender.send((part + 1, 0.0)).await;
                    }
                }
                iso_file_buf.flush().await?;
            }
            let sha256sum = hasher.finalize();
            if let Some(orig_sum) = s.sha256 {
                let orig_sum = hex::decode(orig_sum).context("Could not decode checksum")?;
                if sha256sum.as_slice() != orig_sum {
                    return Err(anyhow!("Checksums do not match"));
                };
            };
            Ok(())
        })
    }
}
