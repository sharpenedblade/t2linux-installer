use crate::ui::app::App;
use iced::{Application, Settings};

mod ui {
    pub mod app;
    pub mod install_page;
    pub mod main_page;
}
mod diskutil;
mod distro;
mod error;
mod install;
mod macos;

fn main() -> iced::Result {
    App::run(Settings::default())
}
