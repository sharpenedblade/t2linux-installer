use std::{path::PathBuf, str::FromStr, sync::Arc};

use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::{
    Color, Length, Task,
    widget::{button, column, container, radio, scrollable, text},
};

use super::finish_page::FinishPage;

#[derive(Debug, Clone)]
pub enum MainPageMessage {
    LoadDistroList(Vec<Distro>),
    LoadBlockDeviceList(Vec<String>),
    Err(Arc<anyhow::Error>),
    PickDistro(usize),
    OpenTargetPicker,
    TriggerFilePicker,
    PickIsoFile(PathBuf),
    PickBlockDeviceIndex(usize),
    StartInstall,
    Ignore,
}

enum MainPageState {
    Distro,
    Target,
}

#[derive(Clone)]
enum DownloadTarget {
    BlockDev(usize),
    File(PathBuf),
}

pub struct MainPage {
    state: MainPageState,
    distro_list: Option<Vec<Distro>>,
    block_dev_list: Option<Vec<String>>,
    distro_index: Option<usize>,
    download_target: Option<DownloadTarget>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            state: MainPageState::Distro,
            distro_list: None,
            block_dev_list: None,
            distro_index: None,
            download_target: None,
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
                        && let Some(DownloadTarget::File(iso_file)) = self.download_target.clone()
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
                MainPageMessage::PickIsoFile(path_buf) => {
                    self.download_target = Some(DownloadTarget::File(path_buf))
                }
                MainPageMessage::PickBlockDeviceIndex(i) => {
                    self.download_target = Some(DownloadTarget::BlockDev(i))
                }
                MainPageMessage::Ignore => {}
                MainPageMessage::LoadDistroList(distros) => self.distro_list = Some(distros),
                MainPageMessage::Err(error) => {
                    task = iced::Task::none();
                    page = Some(Box::new(FinishPage::new(
                        super::finish_page::FinishState::Error(error),
                    )))
                }
                MainPageMessage::OpenTargetPicker => {
                    self.state = MainPageState::Target;
                }
                MainPageMessage::LoadBlockDeviceList(items) => self.block_dev_list = Some(items),
            }
        }
        if self.distro_list.is_none() {
            task = Task::batch([task, get_distro_list()]);
        }
        (page, task)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let e = match self.state {
            MainPageState::Distro => self.distro_picker_view(),
            MainPageState::Target => self.target_picker_view(),
        };
        container(e)
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
        Task::batch([get_distro_list(), get_block_dev_list()])
    }
    fn distro_picker_view(&self) -> iced::widget::Column<'_, AppMessage> {
        let mut distro_list = column![].spacing(16);
        if let Some(distros) = &self.distro_list {
            for (i, distro) in distros.iter().enumerate() {
                distro_list =
                    distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                        AppMessage::Main(MainPageMessage::PickDistro(i))
                    }));
            }
        }
        column![
            text("Choose a distro").size(24),
            scrollable(distro_list).height(400).width(400),
            button("Next").on_press(AppMessage::Main(MainPageMessage::OpenTargetPicker))
        ]
        .spacing(16)
        .padding(8)
    }
    fn target_picker_view(&self) -> iced::widget::Column<'_, AppMessage> {
        column![
            self.file_path_view(),
            self.block_dev_view(),
            button("Begin Download").on_press_maybe(if self.download_target.is_some() {
                Some(AppMessage::Main(MainPageMessage::StartInstall))
            } else {
                None
            })
        ]
        .spacing(32)
        .padding(8)
    }
    fn file_path_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let mut col = column![text("Download to a file").size(24),];
        if let Some(DownloadTarget::File(path)) = &self.download_target {
            col = col.push(text(format!("{}", path.display())));
        } else {
            col = col.push(text("No download path selected"))
        }
        col = col.push(
            button("Choose file path ")
                .on_press(AppMessage::Main(MainPageMessage::TriggerFilePicker)),
        );
        container(col.spacing(16)).width(350)
    }
    fn block_dev_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let selected_i = if let Some(DownloadTarget::BlockDev(n)) = self.download_target {
            Some(n)
        } else {
            None
        };
        let mut list = column![].spacing(16);
        if let Some(devs) = &self.block_dev_list {
            for (cur_i, dev) in devs.iter().enumerate() {
                list = list.push(radio(dev.clone(), cur_i, selected_i, |_| {
                    AppMessage::Main(MainPageMessage::PickBlockDeviceIndex(cur_i))
                }));
            }
        }
        container(column![text("Flash to a disk").size(24), list,].spacing(16)).width(350)
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

fn get_block_dev_list() -> Task<AppMessage> {
    Task::future(async { Ok(vec!["sda".to_owned(), "sdb".to_owned()]) }).then(|handle| match handle
    {
        Ok(list) => Task::done(AppMessage::Main(MainPageMessage::LoadBlockDeviceList(list))),
        Err(e) => Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(e)))),
    })
}
