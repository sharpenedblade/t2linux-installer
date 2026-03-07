use crate::{
    install::{InstallProgress, InstallSettings},
    ui::app::{AppMessage, Page},
    ui::finish_page,
};
use anyhow::anyhow;
use futures::StreamExt;
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, progress_bar, row, text};
use rfd::{MessageButtons, MessageDialogResult, MessageLevel};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::sync::CancellationToken;

use super::finish_page::FinishState;

#[derive(Debug)]
pub struct DownloadPage {
    settings: InstallSettings,
    progress: f64,
    total_parts: Option<usize>,
    current_parts: Option<usize>,
    ct: CancellationToken,
    file: Arc<File>,
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
    CancelDecision(MessageDialogResult),
}

impl DownloadPage {
    pub fn new(settings: InstallSettings, file: File) -> Self {
        Self {
            total_parts: None,
            current_parts: None,
            progress: 0.0,
            settings,
            ct: CancellationToken::new(),
            file: Arc::new(file),
        }
    }
}

impl Page for DownloadPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>) {
        let mut command: iced::Task<AppMessage> = iced::Task::none();
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::Download(msg) = message {
            match msg {
                DownloadPageMessage::StartedIsoDownload(parts) => self.total_parts = Some(parts),
                DownloadPageMessage::Cancel => {
                    command = cancel_confirmation_dialog();
                }
                DownloadPageMessage::CancelDecision(choice) => {
                    if matches!(choice, MessageDialogResult::Yes | MessageDialogResult::Ok) {
                        self.ct.cancel();
                    }
                }
                DownloadPageMessage::Finished => {
                    page = Some(Box::new(finish_page::FinishPage::new(FinishState::Clean {
                        usb_flashed: self.settings.is_block_device_target(),
                    })))
                }
                DownloadPageMessage::Failed(e) => {
                    let state = if self.ct.is_cancelled() {
                        FinishState::Cancelled
                    } else {
                        FinishState::Error(Arc::new(anyhow!(e)))
                    };
                    page = Some(Box::new(finish_page::FinishPage::new(state)))
                }
                DownloadPageMessage::DownloadProgress(_, _) => {}
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<'_, AppMessage> {
        let mut row1 = row![text("Downloading the T2 Linux Image").size(30)]
            .spacing(20)
            .align_y(Vertical::Center);
        if let Some(total_parts) = self.total_parts
            && total_parts > 1
            && let Some(current_parts) = self.current_parts
        {
            row1 = row1.push(text(format!("Part {current_parts} of {total_parts}")).size(18))
        }
        let mut col = column![
            text("Please keep this window open until the download completes.")
                .size(16)
        ]
        .spacing(18);
        col = col.push(
            row![
                text(format!("{:.1}%", self.progress * 100.0))
                    .size(18)
                    .width(64),
                progress_bar(0.0..=100.0, self.progress as f32 * 100.0),
            ]
            .width(460)
            .spacing(18)
            .align_y(Vertical::Center),
        );
        col = col.push(
            row![button("Cancel Download")
                .on_press(AppMessage::Download(DownloadPageMessage::Cancel))]
            .spacing(12),
        );
        container(
            column![row1, col]
                .spacing(26)
                .padding(28)
                .max_width(560),
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
            file: self.file.clone(),
        };
        iced::Subscription::run_with(init, DownloadSubState::subscription_task)
    }

    fn block_window_close(&self) -> bool {
        self.total_parts.is_some() && !self.ct.is_cancelled()
    }
}

#[derive(Debug)]
pub struct DownloadSubState {
    settings: InstallSettings,
    ct: CancellationToken,
    file: Arc<File>,
}

impl Hash for DownloadSubState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.hash(state);
    }
}
impl DownloadSubState {
    fn subscription_task(&self) -> impl futures::Stream<Item = AppMessage> + use<> {
        self.settings
            .install(self.file.clone(), self.ct.clone())
            .map(|msg| match msg {
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

fn cancel_confirmation_dialog() -> iced::Task<AppMessage> {
    iced::Task::future(
        rfd::AsyncMessageDialog::new()
            .set_title("Cancel download?")
            .set_description("Do you want to cancel the current download?")
            .set_level(MessageLevel::Warning)
            .set_buttons(MessageButtons::YesNo)
            .show(),
    )
    .then(|choice| {
        iced::Task::done(AppMessage::Download(DownloadPageMessage::CancelDecision(
            choice,
        )))
    })
}
