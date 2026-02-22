use crate::ui::{download_page, finish_page, main_page};
use iced::{executor, Application, Command};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppMessage {
    MainPage(main_page::MainPageMessage),
    DownloadPage(download_page::DownloadPageMessage),
    FinishPage(finish_page::FinishPageMessage),
}

pub struct App {
    page: Box<dyn Page>,
}

pub trait Page {
    fn update(&mut self, message: AppMessage)
        -> (Option<Box<dyn Page>>, iced::Command<AppMessage>);
    fn view(&self) -> iced::Element<AppMessage>;
    fn subscription(&self) -> iced::Subscription<AppMessage>;
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
        let (page, command) = self.page.update(message);
        if let Some(p) = page {
            self.page = p;
        }
        command
    }

    fn view(&self) -> iced::Element<Self::Message> {
        self.page.view()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        self.page.subscription()
    }
}
