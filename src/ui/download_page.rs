use crate::install::{DownloadSettings, InstallProgress};
use crate::ui::app::{AppMessage, Page};
use crate::ui::finish_page;
use futures::StreamExt;
use iced::widget::{column, container, text};
use iced::Length;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DownloadPage {
    settings: DownloadSettings,
    state: DownloadState,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DownloadState {
    Downloading,
    Failed(String),
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadPageMessage {
    StartedIsoDownload,
    Finished,
    Failed(String),
}

impl DownloadPage {
    pub fn new(settings: DownloadSettings) -> Self {
        Self {
            settings,
            state: DownloadState::Downloading,
        }
    }
}

impl Page for DownloadPage {
    fn update(
        &mut self,
        message: AppMessage,
    ) -> (Option<Box<(dyn Page)>>, iced::Command<AppMessage>) {
        let command: iced::Command<AppMessage> = iced::Command::none();
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::DownloadPage(msg) = message {
            match msg {
                DownloadPageMessage::StartedIsoDownload => {
                    self.state = DownloadState::Downloading;
                }
                DownloadPageMessage::Finished => {
                    page = Some(Box::new(finish_page::FinishPage::new()))
                }
                DownloadPageMessage::Failed(err_msg) => self.state = DownloadState::Failed(err_msg),
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        container(match &self.state {
            DownloadState::Downloading => {
                column![text("Downloading ISO").size(24), text("Please wait...")].spacing(16)
            }
            DownloadState::Finished => column![
                text("Ready for installation!").size(24),
                text("You can reboot to the linux installer now.")
            ]
            .spacing(16),
            DownloadState::Failed(err_msg) => column![
                text("Installation failed").size(24),
                text(format!(
                    "Error: {err_msg}. Please try again or file a bug report"
                ))
            ]
            .spacing(16),
        })
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        let install: iced::Subscription<AppMessage> = iced::subscription::run_with_id(
            0,
            self.settings.install().map(|msg| match msg {
                InstallProgress::DownloadingIso => {
                    AppMessage::DownloadPage(DownloadPageMessage::StartedIsoDownload)
                }
                InstallProgress::Finished => {
                    AppMessage::DownloadPage(DownloadPageMessage::Finished)
                }
                InstallProgress::Failed(err) => {
                    AppMessage::DownloadPage(DownloadPageMessage::Failed(err.to_string()))
                }
            }),
        );
        let mut subscriptions: Vec<iced::Subscription<AppMessage>> = vec![];
        if self.state != DownloadState::Finished {
            subscriptions.push(install)
        }
        iced::Subscription::batch(subscriptions)
    }
}
