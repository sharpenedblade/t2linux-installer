use crate::ui::app::App;
use iced::{Size, window};

mod ui {
    pub mod app;
    pub mod download_page;
    pub mod finish_page;
    pub mod main_page;
}
pub mod disk;
mod distro;
mod error;
mod install;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .exit_on_close_request(false)
        .window(window::Settings {
            size: Size::new(820.0, 560.0),
            min_size: Some(Size::new(640.0, 460.0)),
            ..window::Settings::default()
        })
        .centered()
        .subscription(App::subscription)
        .run()
}
