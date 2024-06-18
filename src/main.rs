use iced::{widget::text, Sandbox, Settings};

struct App {}

#[derive(Debug, Clone, Eq, PartialEq)]
enum AppMessage {}

impl Sandbox for App {
    type Message = AppMessage;

    fn new() -> Self {
        Self {}
    }

    fn title(&self) -> String {
        String::from("t2linux Installer")
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&self) -> iced::Element<Self::Message> {
        text("Hello").into()
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
