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
    iso: String,
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
        let mut request: reqwest::blocking::Response =
            reqwest::blocking::get(&self.distro.iso).unwrap();
        let mut iso_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("download.iso")
            .unwrap();
        request.copy_to(&mut iso_file).unwrap();
        iso_file
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
