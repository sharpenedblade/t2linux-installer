use crate::install::{InstallProgress, InstallSettings};
use crate::ui::app::{AppMessage, Page};
use crate::ui::finish_page;
use anyhow::anyhow;
use futures::StreamExt;
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, text};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use super::finish_page::FinishState;

#[derive(Debug, Clone)]
pub struct DownloadPage {
    settings: InstallSettings,
    progress: f64,
    total_parts: Option<usize>,
    current_parts: Option<usize>,
    ct: CancellationToken,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadPageMessage {
    /// total parts
    StartedIsoDownload(usize),
    /// (part, progress)
    DownloadProgress(usize, f64),
    Finished,
    Failed(String),
    Cancel,
}

impl DownloadPage {
    pub fn new(settings: InstallSettings) -> Self {
        Self {
            total_parts: None,
            current_parts: None,
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
                DownloadPageMessage::StartedIsoDownload(parts) => self.total_parts = Some(parts),
                DownloadPageMessage::Cancel => {
                    self.ct.cancel();
                }
                DownloadPageMessage::Finished => {
                    page = Some(Box::new(finish_page::FinishPage::new(FinishState::Clean)))
                }
                DownloadPageMessage::Failed(e) => {
                    let state = if self.ct.is_cancelled() {
                        FinishState::Cancelled
                    } else {
                        FinishState::Error(Arc::new(anyhow!(e)))
                    };
                    page = Some(Box::new(finish_page::FinishPage::new(state)))
                }
                DownloadPageMessage::DownloadProgress(part, progress) => {
                    self.current_parts = Some(part);
                    self.progress = progress
                }
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut row1 = row![text("Downloading ISO").size(24)]
            .spacing(16)
            .align_y(Vertical::Center);
        if let Some(total_parts) = self.total_parts
            && total_parts > 1
            && let Some(current_parts) = self.current_parts
        {
            row1 = row1.push(text(format!("Part {current_parts} of {total_parts}")))
        }
        let mut col = column![row1,].spacing(16);
        col = col.push(
            row![
                text(format!("{:.2}%", self.progress * 100.0)),
                progress_bar(0.0..=100.0, self.progress as f32 * 100.0),
            ]
            .width(400)
            .spacing(16)
            .align_y(Vertical::Center),
        );
        col =
            col.push(button("Cancel").on_press(AppMessage::Download(DownloadPageMessage::Cancel)));
        container(col)
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
    fn subscription_task(&self) -> impl futures::Stream<Item = AppMessage> + use<> {
        self.settings.install(self.ct.clone()).map(|msg| match msg {
            InstallProgress::IsoDownloadStart(parts) => {
                AppMessage::Download(DownloadPageMessage::StartedIsoDownload(parts))
            }
            InstallProgress::IsoDownloadProgress(part, progress) => {
                AppMessage::Download(DownloadPageMessage::DownloadProgress(part, progress))
            }
            InstallProgress::Finished => AppMessage::Download(DownloadPageMessage::Finished),
            InstallProgress::Failed(err) => {
                println!("{err:#}");
                AppMessage::Download(DownloadPageMessage::Failed(format!("{err:#}")))
            }
        })
    }
}
