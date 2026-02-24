use std::{path::PathBuf, sync::Arc};

use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::{
    Length, Task,
    widget::{button, column, container, radio, scrollable, text},
};

use super::finish_page::FinishPage;

#[derive(Debug, Clone)]
pub enum MainPageMessage {
    LoadDistroList(Vec<Distro>),
    Err(Arc<anyhow::Error>),
    PickDistro(usize),
    TriggerFilePicker,
    PickIsoFile(PathBuf),
    StartInstall,
    Ignore,
}

pub struct MainPage {
    distro_list: Option<Vec<Distro>>,
    distro_index: Option<usize>,
    iso_file: Option<PathBuf>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            distro_list: None,
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
                        && let Some(distro_list) = self.distro_list.clone()
                    {
                        let install_settings = InstallSettings::new(
                            distro_list.get(distro_index).unwrap().clone(),
                            iso_file,
                        );
                        page = Some(Box::new(download_page::DownloadPage::new(install_settings)))
                    }
                }
                MainPageMessage::TriggerFilePicker => {
                    task = open_file(if let Some(i) = self.distro_index {
                        format!(
                            "{}.iso",
                            self.distro_list
                                .clone()
                                .and_then(|l| l.get(i).cloned())
                                .map(|d| d.name.clone())
                                .unwrap_or("unknown".to_string())
                        )
                    } else {
                        "unknown.iso".to_owned()
                    });
                }
                MainPageMessage::PickIsoFile(path_buf) => self.iso_file = Some(path_buf),
                MainPageMessage::Ignore => {}
                MainPageMessage::LoadDistroList(distros) => self.distro_list = Some(distros),
                MainPageMessage::Err(error) => {
                    task = iced::Task::none();
                    page = Some(Box::new(FinishPage::new(
                        super::finish_page::FinishState::Error(error),
                    )))
                }
            }
        }
        if self.distro_list.is_none() {
            task = Task::batch([task, get_distro_list()]);
        }
        (page, task)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut distro_list = column![].spacing(16);
        if let Some(distros) = &self.distro_list {
            for (i, distro) in distros.iter().enumerate() {
                distro_list =
                    distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                        AppMessage::Main(MainPageMessage::PickDistro(i))
                    }));
            }
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

impl MainPage {
    pub fn init_tasks() -> Task<AppMessage> {
        get_distro_list()
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
fn get_distro_list() -> Task<AppMessage> {
    Task::future(Distro::get_all()).then(|handle| match handle {
        Ok(distros) => Task::done(AppMessage::Main(MainPageMessage::LoadDistroList(distros))),
        Err(e) => Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(e)))),
    })
}
