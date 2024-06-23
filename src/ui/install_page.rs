use crate::ui::app::{AppMessage, Page};
use crate::InstallSettings;
use iced::widget::{button, column, container, row, text};
use iced::Length;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InstallPage {
    install_settings: InstallSettings,
    state: InstallState,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum InstallState {
    NotStarted,
    DownloadingIso,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPageMessage {
    StartInstallation,
    IsoDownloadEnd,
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
        let mut command: iced::Command<AppMessage> = iced::Command::none();
        if let AppMessage::InstallPage(msg) = message {
            match msg {
                InstallPageMessage::StartInstallation => {
                    self.state = InstallState::DownloadingIso;
                    let s: Self = self.clone();
                    command = iced::Command::perform(
                        async move {
                            s.install_settings.install().await.unwrap();
                        },
                        |_| AppMessage::InstallPage(InstallPageMessage::IsoDownloadEnd),
                    )
                }
                InstallPageMessage::IsoDownloadEnd => self.state = InstallState::Done,
            }
        }
        (None, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        container(match self.state {
            InstallState::NotStarted => {
                column![
                    text("Start Installation").size(24),
                    text("This will permanently and irreversably modify your system. Make sure you have selected the right disk for the ISO. You will not be able to recover your data after clicking start."),
                    button("Start Installation").on_press(AppMessage::InstallPage(
                        InstallPageMessage::StartInstallation
                    ))
                ].spacing(16).width(600)
            }
            InstallState::DownloadingIso => {
                column![text("Downloading ISO").size(24), text("Please wait...")].spacing(16)
            }
            InstallState::Done => {
                column![
                    text("Ready for installation!").size(24),
                    text("You can reboot to the linux installer now.")
                ].spacing(16)
            }
        })
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
