use crate::install::{DownloadSettings, InstallProgress};
use crate::ui::app::{AppMessage, Page};
use crate::ui::finish_page;
use futures::StreamExt;
use iced::widget::{button, column, container, text};
use iced::Length;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct DownloadPage {
    settings: DownloadSettings,
    state: DownloadState,
    progress: f64,
    ct: CancellationToken,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DownloadState {
    Downloading,
    Cancelled,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadPageMessage {
    StartedIsoDownload,
    DownloadProgress(f64),
    Finished,
    Failed(String),
    Cancel,
}

impl DownloadPage {
    pub fn new(settings: DownloadSettings) -> Self {
        Self {
            progress: 0.0,
            settings,
            state: DownloadState::Downloading,
            ct: CancellationToken::new(),
        }
    }
}

impl Page for DownloadPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<(dyn Page)>>, iced::Task<AppMessage>) {
        let command: iced::Task<AppMessage> = iced::Task::none();
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::Download(msg) = message {
            match msg {
                DownloadPageMessage::StartedIsoDownload => {
                    self.state = DownloadState::Downloading;
                }
                DownloadPageMessage::Cancel => {
                    self.ct.cancel();
                }
                DownloadPageMessage::Finished => {
                    page = Some(Box::new(finish_page::FinishPage::new()))
                }
                DownloadPageMessage::Failed(err_msg) => {
                    if self.ct.is_cancelled() {
                        self.state = DownloadState::Cancelled
                    } else {
                        self.state = DownloadState::Failed(err_msg)
                    }
                }
                DownloadPageMessage::DownloadProgress(progress) => self.progress = progress,
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        container(match &self.state {
            DownloadState::Downloading => column![
                text("Downloading ISO").size(24),
                text("Please wait..."),
                button("Cancel").on_press(AppMessage::Download(DownloadPageMessage::Cancel))
            ]
            .spacing(16),
            DownloadState::Cancelled => column![text("Download cancelled").size(24),].spacing(16),
            DownloadState::Failed(err_msg) => column![
                text("Download failed").size(24),
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
        let install: iced::Subscription<AppMessage> = iced::Subscription::run_with(
            &self,
            DownloadPage::subscription_task
                as (fn(&DownloadPage) -> impl futures::Stream<Item = AppMessage>),
        );
        let subscriptions: Vec<iced::Subscription<AppMessage>> = vec![install];
        iced::Subscription::batch(subscriptions)
    }
}
impl DownloadPage {
    fn subscription_task(&self) -> impl futures::Stream<Item = AppMessage> {
        self.settings.install(self.ct.clone()).map(|msg| match msg {
            InstallProgress::IsoDownloadStart => {
                AppMessage::Download(DownloadPageMessage::StartedIsoDownload)
            }
            InstallProgress::IsoDownloadProgress(progress) => {
                AppMessage::Download(DownloadPageMessage::DownloadProgress(progress))
            }
            InstallProgress::Finished => AppMessage::Download(DownloadPageMessage::Finished),
            InstallProgress::Failed(err) => {
                AppMessage::Download(DownloadPageMessage::Failed(err.to_string()))
            }
        })
    }
}
