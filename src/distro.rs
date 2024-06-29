use anyhow::Result;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Distro {
    pub name: String,
    iso_compression: Option<CompressionAlgorithim>,
    iso: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
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

    pub async fn download_iso(&self) -> Result<fs::File> {
        let client = reqwest::Client::new();
        fs::remove_file("download.iso").ok();
        let mut iso_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("download.iso")?;
        for url in &self.iso {
            let mut request = client.get(url).send().await?.bytes_stream();
            while let Some(Ok(data)) = request.next().await {
                iso_file.write_all(&data)?;
            }
        }
        if let Some(compression_algo) = &self.iso_compression {
            match compression_algo {
                CompressionAlgorithim::Zip => {
                    let mut archive = zip::ZipArchive::new(iso_file.try_clone().unwrap())?;
                    let mut decompressed_file = archive.by_index(0)?;
                    std::io::copy(&mut decompressed_file, &mut iso_file)?;
                }
            }
        }
        Ok(fs::OpenOptions::new().read(true).open("download.iso")?)
    }
}
