use crate::ui::app::App;
use iced::{Sandbox, Settings};

mod ui {
    pub mod app;
    pub mod main_page;
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
enum Distro {
    Arch,
    Fedora,
    Ubuntu,
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
