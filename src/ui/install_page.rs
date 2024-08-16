use crate::install::{InstallProgress, InstallSettings};
use crate::ui::app::{AppMessage, Page};
use futures::StreamExt;
use iced::widget::{button, column, container, text};
use iced::Length;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InstallPage {
    install_settings: InstallSettings,
    state: InstallState,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum InstallState {
    NotStarted,
    Starting,
    DownloadingIso,
    FlashingIso,
    ResizingMacos,
    Failed(String),
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPageMessage {
    StartInstallation,
    StartedIsoDownload,
    StartedIsoFlash,
    StartedMacosResize,
    Finished,
    Failed(String),
}

impl InstallPage {
    pub fn new(install_settings: InstallSettings) -> Self {
        Self {
            install_settings,
            state: InstallState::NotStarted,
        }
    }
}

impl Page for InstallPage {
    fn update(
        &mut self,
        message: AppMessage,
    ) -> (Option<Box<(dyn Page)>>, iced::Command<AppMessage>) {
        let command: iced::Command<AppMessage> = iced::Command::none();
        if let AppMessage::InstallPage(msg) = message {
            match msg {
                InstallPageMessage::StartInstallation => self.state = InstallState::Starting,
                InstallPageMessage::StartedIsoDownload => {
                    self.state = InstallState::DownloadingIso;
                }
                InstallPageMessage::StartedIsoFlash => {
                    self.state = InstallState::FlashingIso;
                }
                InstallPageMessage::StartedMacosResize => {
                    self.state = InstallState::ResizingMacos;
                }
                InstallPageMessage::Finished => self.state = InstallState::Finished,
                InstallPageMessage::Failed(err_msg) => self.state = InstallState::Failed(err_msg),
            }
        }
        (None, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        container(match &self.state {
            InstallState::NotStarted => {
                column![
                    text("Start Installation").size(24),
                    text("This will permanently and irreversably modify your system. Make sure you have selected the right disk for the ISO. You will not be able to recover your data after clicking start."),
                    button("Start Installation").on_press(AppMessage::InstallPage(
                        InstallPageMessage::StartInstallation
                    ))
                ].spacing(16).width(600)
            },
            InstallState::Starting => {
                column![text("Starting Installer").size(24), text("If this takes more than a few secods, something is broken")].spacing(16)
            }
            InstallState::DownloadingIso  => {
                column![text("Downloading ISO").size(24), text("Please wait...")].spacing(16)
            }
            InstallState::FlashingIso => {
                column![text("Flashing ISO").size(24), text("Please wait...")].spacing(16)
            },
            InstallState::ResizingMacos => {
                column![text("Resizing MacOS partition").size(24), text("Please wait...")].spacing(16)
            },
            InstallState::Finished => {
                column![
                    text("Ready for installation!").size(24),
                    text("You can reboot to the linux installer now.")
                ].spacing(16)
            }
            InstallState::Failed(err_msg) => {
                column![
                    text("Installation failed").size(24),
                    text(format!("Error: {err_msg}. Please try again or file a bug report"))
                ].spacing(16)
            },
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
            self.install_settings.install().map(|msg| match msg {
                InstallProgress::Started => {
                    AppMessage::InstallPage(InstallPageMessage::StartedIsoDownload)
                }
                InstallProgress::DownloadedIso => {
                    AppMessage::InstallPage(InstallPageMessage::StartedIsoFlash)
                }
                InstallProgress::ResizingMacos => {
                    AppMessage::InstallPage(InstallPageMessage::StartedMacosResize)
                }
                InstallProgress::Finished => AppMessage::InstallPage(InstallPageMessage::Finished),
                InstallProgress::Failed(err) => {
                    AppMessage::InstallPage(InstallPageMessage::Failed(err.to_string()))
                }
            }),
        );
        let mut subscriptions: Vec<iced::Subscription<AppMessage>> = vec![];
        if self.state != InstallState::NotStarted && self.state != InstallState::Finished {
            subscriptions.push(install)
        }
        iced::Subscription::batch(subscriptions)
    }
}
