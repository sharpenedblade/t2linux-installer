use crate::ui::{install_page, main_page};
use iced::{executor, Application, Command};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppMessage {
    MainPage(main_page::MainPageMessage),
    InstallPage(install_page::InstallPageMessage),
}

pub struct App {
    page: Box<dyn Page>,
}

pub trait Page {
    fn update(&mut self, message: AppMessage) -> Option<Box<dyn Page>>;
    fn view(&self) -> iced::Element<AppMessage>;
}

impl Application for App {
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = ();
    type Message = AppMessage;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                page: Box::new(main_page::MainPage::new()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("t2linux Installer")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        if let Some(p) = self.page.update(message) {
            self.page = p;
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        self.page.view()
    }
}
