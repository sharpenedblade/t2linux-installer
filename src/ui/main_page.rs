use crate::diskutil::{self, GIGABYTE};
use crate::ui::{
    app::{AppMessage, Page},
    install_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::widget::{button, checkbox, column, radio, slider, text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainPageMessage {
    PickDistro(usize),
    ToggleAutoPartitioning(bool),
    PickDisk(String),
    StartInstall,
    ChangeMacosSize(u64),
}

pub struct MainPage {
    distro_index: Option<usize>,
    shrink_macos: bool,
    macos_size: u64,
    target_disk: Option<String>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            distro_index: None,
            shrink_macos: true,
            target_disk: None,
            macos_size: 0,
        }
    }
}

impl Page for MainPage {
    fn update(
        &mut self,
        message: AppMessage,
    ) -> (Option<Box<dyn Page>>, iced::Command<AppMessage>) {
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::MainPage(msg) = message {
            match msg {
                MainPageMessage::PickDistro(distro_index) => self.distro_index = Some(distro_index),
                MainPageMessage::ToggleAutoPartitioning(b) => self.shrink_macos = b,
                MainPageMessage::PickDisk(s) => self.target_disk = Some(s),
                MainPageMessage::StartInstall => {
                    let install_settings = InstallSettings::new(
                        Distro::get_all()
                            .unwrap()
                            .get(self.distro_index.unwrap())
                            .unwrap()
                            .clone(),
                        self.target_disk.clone().unwrap(),
                        if self.shrink_macos {
                            Some(self.macos_size)
                        } else {
                            None
                        },
                    );
                    page = Some(Box::new(install_page::InstallPage::new(install_settings)))
                }
                MainPageMessage::ChangeMacosSize(i) => {
                    self.macos_size = i;
                }
            }
        }
        (page, iced::Command::none())
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut disk_list = column![text("Choose instalation disk").size(24)].spacing(4);
        let disks = diskutil::get_external_disks();
        if disks.is_empty() {
            disk_list = disk_list.push(text(
                "No external disks found. Please connect a disk and try again",
            ));
        } else {
            for disk in &disks {
                disk_list = disk_list.push(radio(
                    disk,
                    &disk.to_string(),
                    self.target_disk.as_ref(),
                    |s| AppMessage::MainPage(MainPageMessage::PickDisk(s.into())),
                ));
            }
        }
        let mut distro_list = column![text("Choose a distro").size(24),].spacing(8);
        let distros = Distro::get_all().unwrap();
        for (i, distro) in distros.iter().enumerate() {
            distro_list =
                distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                    AppMessage::MainPage(MainPageMessage::PickDistro(i))
                }));
        }
        let (min_size, max_size) = diskutil::get_resize_limits("disk0s2");

        let mut macos_shrink = column![checkbox("Shrink MacOS", self.shrink_macos)
            .on_toggle(|b| AppMessage::MainPage(MainPageMessage::ToggleAutoPartitioning(b))),]
        .spacing(16);
        if self.shrink_macos {
            if min_size < max_size && max_size - min_size > GIGABYTE {
                macos_shrink = macos_shrink.push(
                    column![
                        text(format!(
                            "MacOS partition size: {:.2} GB",
                            (self.macos_size as f64 / GIGABYTE as f64)
                        )),
                        slider(
                            min_size as f64..=max_size as f64,
                            self.macos_size as f64,
                            |f| {
                                AppMessage::MainPage(MainPageMessage::ChangeMacosSize(f as u64))
                            },
                        )
                        .step(GIGABYTE as f64)
                        .width(iced::Length::Fixed(400.0)),
                    ]
                    .spacing(4),
                );
            } else {
                macos_shrink = macos_shrink.push(text(
                    "MacOS partition is full, please create some free space",
                ))
            }
        }
        column![
            distro_list,
            disk_list,
            macos_shrink,
            button("Begin installation")
                .on_press(AppMessage::MainPage(MainPageMessage::StartInstall))
        ]
        .spacing(16)
        .padding(8)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::none()
    }
}
