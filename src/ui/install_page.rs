use crate::ui::app::{AppMessage, Page};
use crate::InstallSettings;
use iced::widget::{button, column};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InstallPage {
    install_settings: InstallSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPageMessage {
    StartInstallation,
    IsoDownloadEnd,
}

impl InstallPage {
    pub fn new(install_settings: InstallSettings) -> Self {
        Self { install_settings }
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
                    let s: Self = self.clone();
                    command = iced::Command::perform(
                        async move {
                            s.install_settings.install().await;
                        },
                        |_| AppMessage::InstallPage(InstallPageMessage::IsoDownloadEnd),
                    )
                }
                InstallPageMessage::IsoDownloadEnd => {}
            }
        }
        (None, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        column![
            button("Start Installation").on_press(AppMessage::InstallPage(
                InstallPageMessage::StartInstallation
            ))
        ]
        .into()
    }
}
