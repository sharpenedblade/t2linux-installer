use crate::ui::{download_page, finish_page, main_page};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMessage {
    Main(main_page::MainPageMessage),
    Download(download_page::DownloadPageMessage),
    Finish(finish_page::FinishPageMessage),
}

pub struct App {
    page: Box<dyn Page>,
}

pub trait Page {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>);
    fn view(&self) -> iced::Element<AppMessage>;
    fn subscription(&self) -> iced::Subscription<AppMessage>;
}

impl App {
    pub fn new(_flags: ()) -> (Self, iced::Task<AppMessage>) {
        (
            Self {
                page: Box::new(main_page::MainPage::new()),
            },
            iced::Task::none(),
        )
    }

    pub fn title(&self) -> String {
        String::from("t2linux Installer")
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        let (page, command) = self.page.update(message);
        if let Some(p) = page {
            self.page = p;
        }
        command
    }

    pub fn view(&self) -> iced::Element<AppMessage> {
        self.page.view()
    }

    pub fn subscription(&self) -> iced::Subscription<AppMessage> {
        self.page.subscription()
    }
}
