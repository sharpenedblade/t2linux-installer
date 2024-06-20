use crate::ui::{
    app::{AppMessage, Page},
    install_page,
};
use crate::{Distro, InstallSettings};
use iced::widget::{button, checkbox, column, radio, text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainPageMessage {
    PickDistro(usize),
    ToggleAutoPartitioning(bool),
    PickDisk(String),
    StartInstall,
}

pub struct MainPage {
    distro_index: Option<usize>,
    auto_partitioning: bool,
    target_disk: Option<String>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            distro_index: None,
            auto_partitioning: true,
            target_disk: None,
        }
    }
}

impl Page for MainPage {
    fn update(&mut self, message: AppMessage) -> Option<Box<dyn Page>> {
        if let AppMessage::MainPage(msg) = message {
            match msg {
                MainPageMessage::PickDistro(distro_index) => self.distro_index = Some(distro_index),
                MainPageMessage::ToggleAutoPartitioning(b) => self.auto_partitioning = b,
                MainPageMessage::PickDisk(s) => self.target_disk = Some(s),
                MainPageMessage::StartInstall => {
                    let install_settings = InstallSettings {
                        distro: Distro::get_all()[self.distro_index.unwrap()].clone(),
                    };
                    return Some(Box::new(install_page::InstallPage::new(install_settings)));
                }
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
        let mut distro_list = column![text("Choose a distro").size(24),].spacing(8);
        let distros = Distro::get_all();
        for (i, distro) in distros.iter().enumerate() {
            distro_list =
                distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                    AppMessage::MainPage(MainPageMessage::PickDistro(i))
                }));
        }
        column![
            distro_list,
            disk_list,
            checkbox("Automatic partitioning", self.auto_partitioning)
                .on_toggle(|b| AppMessage::MainPage(MainPageMessage::ToggleAutoPartitioning(b))),
            button("Begin installation")
                .on_press(AppMessage::MainPage(MainPageMessage::StartInstall))
        ]
        .spacing(16)
        .padding(8)
        .into()
    }
}
