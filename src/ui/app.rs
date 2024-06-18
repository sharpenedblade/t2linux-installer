use crate::ui::main_page;
use iced::Sandbox;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppMessage {
    MainPage(main_page::MainPageMessage),
}

pub struct App {
    page: Box<dyn Page>,
}

pub trait Page {
    fn update(&mut self, message: AppMessage) -> Option<Box<dyn Page>>;
    fn view(&self) -> iced::Element<AppMessage>;
}

impl Sandbox for App {
    type Message = AppMessage;

    fn new() -> Self {
        Self {
            page: Box::new(main_page::MainPage::new()),
        }
    }

    fn title(&self) -> String {
        String::from("t2linux Installer")
    }

    fn update(&mut self, _message: Self::Message) {
        if let Some(p) = self.page.update(_message) {
            self.page = p;
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        self.page.view()
    }
}
