use crate::ui::app::App;
use anyhow::Result;
use futures::StreamExt;
use iced::{Application, Settings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

mod ui {
    pub mod app;
    pub mod install_page;
    pub mod main_page;
}
mod error;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct Distro {
    name: String,
    iso_compression: Option<CompressionAlgorithim>,
    iso: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
enum CompressionAlgorithim {
    Zip,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct DistroMetadataWrapper {
    all: Vec<Distro>,
}

impl Distro {
    fn get_all() -> Result<Vec<Distro>> {
        let iso_metadata_file = fs::OpenOptions::new()
            .read(true)
            .open("distro-metadata.json")?;
        let iso_metadata: DistroMetadataWrapper = serde_json::from_reader(iso_metadata_file)?;
        Ok(iso_metadata.all)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct InstallSettings {
    distro: Distro,
}

impl InstallSettings {
    async fn install(&self) -> Result<()> {
        let iso_file = self.download_iso().await?;
        Ok(())
    }
    async fn download_iso(&self) -> Result<fs::File> {
        let client = reqwest::Client::new();
        fs::remove_file("download.iso")?;
        let mut iso_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("download.iso")?;
        for url in &self.distro.iso {
            let mut request = client.get(url).send().await?.bytes_stream();
            while let Some(Ok(data)) = request.next().await {
                iso_file.write_all(&data)?;
            }
        }
        if let Some(compression_algo) = &self.distro.iso_compression {
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

fn main() -> iced::Result {
    App::run(Settings::default())
}
