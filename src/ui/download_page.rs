use crate::install::{InstallProgress, InstallSettings};
use crate::ui::app::{AppMessage, Page};
use crate::ui::finish_page;
use futures::StreamExt;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, text};
use iced::Length;
use std::hash::{Hash, Hasher};
use tokio_util::sync::CancellationToken;

use super::finish_page::FinishState;

#[derive(Debug, Clone)]
pub struct DownloadPage {
    settings: InstallSettings,
    progress: f64,
    ct: CancellationToken,
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
    pub fn new(settings: InstallSettings) -> Self {
        Self {
            progress: 0.0,
            settings,
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
                DownloadPageMessage::StartedIsoDownload => {}
                DownloadPageMessage::Cancel => {
                    self.ct.cancel();
                }
                DownloadPageMessage::Finished => {
                    page = Some(Box::new(finish_page::FinishPage::new(FinishState::Clean)))
                }
                DownloadPageMessage::Failed(_) => {
                    let state = if self.ct.is_cancelled() {
                        FinishState::Cancelled
                    } else {
                        FinishState::Error
                    };
                    page = Some(Box::new(finish_page::FinishPage::new(state)))
                }
                DownloadPageMessage::DownloadProgress(progress) => self.progress = progress,
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        container(
            column![
                text("Downloading ISO").size(24),
                row![
                    text(format!("{:.2}%", self.progress)),
                    progress_bar(0.0..=100.0, self.progress as f32 * 100.0),
                ]
                .width(400)
                .spacing(16)
                .align_y(Vertical::Center),
                button("Cancel").on_press(AppMessage::Download(DownloadPageMessage::Cancel))
            ]
            .spacing(16),
        )
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        let init = DownloadSubState {
            settings: self.settings.clone(),
            ct: self.ct.clone(),
        };
        iced::Subscription::run_with(init, DownloadSubState::subscription_task)
    }
}

#[derive(Debug, Clone)]
pub struct DownloadSubState {
    settings: InstallSettings,
    ct: CancellationToken,
}

impl Hash for DownloadSubState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.hash(state);
    }
}
impl DownloadSubState {
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
