use crate::ui::app::{AppMessage, Page};
use crate::Distro;
use iced::widget::{button, checkbox, column, radio, text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainPageMessage {
    PickDistro(Distro),
    ToggleAutoPartitioning(bool),
    PickDisk(String),
}

pub struct MainPage {
    distro: Option<Distro>,
    auto_partitioning: bool,
    target_disk: Option<String>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            distro: None,
            auto_partitioning: true,
            target_disk: None,
        }
    }
}

impl Page for MainPage {
    fn update(&mut self, _message: AppMessage) -> Option<Box<dyn Page>> {
        if let AppMessage::MainPage(msg) = _message {
            match msg {
                MainPageMessage::PickDistro(distro) => self.distro = Some(distro),
                MainPageMessage::ToggleAutoPartitioning(b) => self.auto_partitioning = b,
                MainPageMessage::PickDisk(s) => self.target_disk = Some(s),
            }
        }
        None
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut disk_list = column![text("Choose instalation disk").size(24)].spacing(4);
        for disk in ["disk0", "disk2"] {
            disk_list = disk_list.push(radio(
                disk,
                &disk.to_string(),
                self.target_disk.as_ref(),
                |s| AppMessage::MainPage(MainPageMessage::PickDisk(s.into())),
            ));
        }
        column![
            column![
                text("Choose a distro").size(24),
                radio("Arch Linux", Distro::Arch, self.distro, |d| {
                    AppMessage::MainPage(MainPageMessage::PickDistro(d))
                }),
                radio("Fedora", Distro::Fedora, self.distro, |d| {
                    AppMessage::MainPage(MainPageMessage::PickDistro(d))
                }),
                radio("Ubuntu", Distro::Ubuntu, self.distro, |d| {
                    AppMessage::MainPage(MainPageMessage::PickDistro(d))
                })
            ]
            .spacing(4),
            disk_list,
            checkbox("Automatic partitioning", self.auto_partitioning)
                .on_toggle(|b| AppMessage::MainPage(MainPageMessage::ToggleAutoPartitioning(b))),
            button("Begin installation")
        ]
        .spacing(16)
        .padding(8)
        .into()
    }
}
