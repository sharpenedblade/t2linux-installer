use crate::ui::app::App;
use iced::{Sandbox, Settings};
use std::fs;

mod ui {
    pub mod app;
    pub mod main_page;
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Distro {
    name: String,
    iso: String,
}

impl Distro {
    fn get_all() -> Vec<Distro> {
        vec![
            Distro {
                name: "Arch Linux".to_string(),
                iso: "https://example.com".to_string(),
            },
            Distro {
                name: "Fedora".to_string(),
                iso: "https://example.com".to_string(),
            },
            Distro {
                name: "Ubuntu".to_string(),
                iso: "https://example.com".to_string(),
            },
        ]
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
