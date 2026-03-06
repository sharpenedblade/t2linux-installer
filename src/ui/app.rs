use crate::ui::{download_page, finish_page, main_page};
use iced::window;

#[derive(Debug, Clone)]
pub enum AppMessage {
    Main(main_page::MainPageMessage),
    Download(download_page::DownloadPageMessage),
    Finish(finish_page::FinishPageMessage),
    WindowCloseRequested(window::Id),
}

pub struct App {
    page: Box<dyn Page>,
}

pub trait Page {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>);
    fn view(&self) -> iced::Element<'_, AppMessage>;
    fn subscription(&self) -> iced::Subscription<AppMessage>;
    fn block_window_close(&self) -> bool {
        false
    }
}

impl App {
    pub fn new() -> (Self, iced::Task<AppMessage>) {
        (
            Self {
                page: Box::new(main_page::MainPage::new()),
            },
            main_page::MainPage::init_tasks(),
        )
    }

    pub fn title(&self) -> String {
        String::from("t2linux Installer")
    }

    pub fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        if let AppMessage::WindowCloseRequested(id) = message {
            return if self.page.block_window_close() {
                iced::Task::none()
            } else {
                window::close(id)
            };
        }

        let (page, command) = self.page.update(message);
        if let Some(p) = page {
            self.page = p;
        }
        command
    }

    pub fn view(&self) -> iced::Element<'_, AppMessage> {
        self.page.view()
    }

    pub fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::batch([
            self.page.subscription(),
            window::close_requests().map(AppMessage::WindowCloseRequested),
        ])
    }
}
