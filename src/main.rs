use crate::ui::app::App;
use iced::{Sandbox, Settings};
use serde::{Deserialize, Serialize};
use std::fs;

mod ui {
    pub mod app;
    pub mod main_page;
}

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
    fn get_all() -> Vec<Distro> {
        let iso_metadata_file = fs::OpenOptions::new()
            .read(true)
            .open("distro-metadata.json")
            .unwrap();
        let iso_metadata: DistroMetadataWrapper =
            serde_json::from_reader(iso_metadata_file).unwrap();
        iso_metadata.all
    }
}

struct InstallSettings {
    distro: Distro,
}

impl InstallSettings {
    fn install(&self) {
        let iso_file = self.download_iso();
    }
    fn download_iso(&self) -> fs::File {
        let client = reqwest::blocking::Client::new();
        let mut iso_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("download.iso")
            .unwrap();
        for url in &self.distro.iso {
            let mut request: reqwest::blocking::Response = client.get(url).send().unwrap();
            request.copy_to(&mut iso_file).unwrap();
        }
        if let Some(compression_algo) = &self.distro.iso_compression {
            match compression_algo {
                CompressionAlgorithim::Zip => {
                    let mut archive = zip::ZipArchive::new(iso_file.try_clone().unwrap()).unwrap();
                    let mut decompressed_file = archive.by_index(0).unwrap();
                    std::io::copy(&mut decompressed_file, &mut iso_file).unwrap();
                }
            }
        }
        fs::OpenOptions::new()
            .read(true)
            .open("download.iso")
            .unwrap()
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
