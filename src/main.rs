use crate::ui::app::App;

mod ui {
    pub mod app;
    pub mod download_page;
    pub mod finish_page;
    pub mod main_page;
}
mod distro;
mod error;
mod install;

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| App::new(()))
}
