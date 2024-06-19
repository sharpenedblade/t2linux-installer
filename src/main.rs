use crate::ui::app::App;
use iced::{Sandbox, Settings};

mod ui {
    pub mod app;
    pub mod main_page;
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Distro {
    name: String,
}

impl Distro {
    fn get_all() -> Vec<Distro> {
        vec![
            Distro {
                name: "Arch Linux".to_string(),
            },
            Distro {
                name: "Fedora".to_string(),
            },
            Distro {
                name: "Ubuntu".to_string(),
            },
        ]
    }
}

struct InstallSettings {
    distro: Distro,
}

impl InstallSettings {
    fn install(&self) {
        todo!();
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
