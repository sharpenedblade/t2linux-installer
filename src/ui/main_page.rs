use std::path::PathBuf;

use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::{
    Length, Task,
    widget::{button, column, container, radio, scrollable, text},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainPageMessage {
    PickDistro(usize),
    TriggerFilePicker,
    PickIsoFile(PathBuf),
    StartInstall,
    Ignore,
}

pub struct MainPage {
    distro_index: Option<usize>,
    iso_file: Option<PathBuf>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            distro_index: None,
            iso_file: None,
        }
    }
}

impl Page for MainPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>) {
        let mut page: Option<Box<dyn Page>> = None;
        let mut task = iced::Task::none();
        if let AppMessage::Main(msg) = message {
            match msg {
                MainPageMessage::PickDistro(distro_index) => self.distro_index = Some(distro_index),
                MainPageMessage::StartInstall => {
                    if let Some(distro_index) = self.distro_index
                        && let Some(iso_file) = self.iso_file.clone()
                    {
                        let install_settings = InstallSettings::new(
                            Distro::get_all()
                                .unwrap()
                                .get(distro_index)
                                .unwrap()
                                .clone(),
                            iso_file,
                        );
                        page = Some(Box::new(download_page::DownloadPage::new(install_settings)))
                    }
                }
                MainPageMessage::TriggerFilePicker => {
                    task = open_file(if let Some(i) = self.distro_index {
                        format!(
                            "{}.iso",
                            Distro::get_all()
                                .unwrap()
                                .get(i)
                                .map(|d| d.name.as_str())
                                .unwrap_or("unknown")
                        )
                    } else {
                        "unknown.iso".to_owned()
                    });
                }
                MainPageMessage::PickIsoFile(path_buf) => self.iso_file = Some(path_buf),
                MainPageMessage::Ignore => {}
            }
        }
        (page, task)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut distro_list = column![].spacing(16);
        let distros = Distro::get_all().unwrap();
        for (i, distro) in distros.iter().enumerate() {
            distro_list =
                distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                    AppMessage::Main(MainPageMessage::PickDistro(i))
                }));
        }

        container(
            column![
                text("Choose a distro").size(24),
                scrollable(distro_list).height(400).width(400),
                button("Chose file location")
                    .on_press(AppMessage::Main(MainPageMessage::TriggerFilePicker)),
                button("Next").on_press(AppMessage::Main(MainPageMessage::StartInstall))
            ]
            .spacing(16)
            .padding(8),
        )
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::none()
    }
}

fn open_file(name: String) -> Task<AppMessage> {
    Task::future(
        rfd::AsyncFileDialog::new()
            .add_filter("ISO files", &["iso"])
            .set_file_name(name)
            .save_file(),
    )
    .then(|handle| match handle {
        Some(file_handle) => Task::done(AppMessage::Main(MainPageMessage::PickIsoFile(
            file_handle.into(),
        ))),
        None => Task::done(AppMessage::Main(MainPageMessage::Ignore)),
    })
}
