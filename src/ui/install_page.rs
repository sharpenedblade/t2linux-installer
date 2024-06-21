use crate::ui::app::{AppMessage, Page};
use crate::InstallSettings;
use iced::widget::{column, text};

pub struct InstallPage {
    install_settings: InstallSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPageMessage {}

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
        if let AppMessage::InstallPage(_msg) = message {}
        (None, iced::Command::none())
    }
    fn view(&self) -> iced::Element<AppMessage> {
        column![text("Installing")].into()
    }
}
