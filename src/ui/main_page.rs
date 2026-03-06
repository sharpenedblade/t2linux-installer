use std::{path::PathBuf, sync::Arc};

use crate::disk::{self, BlockDevice};
use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::{
    Length, Task,
    widget::{button, column, container, radio, row, scrollable, text, text_input},
};

use super::finish_page::FinishPage;

#[derive(Debug, Clone)]
pub enum MainPageMessage {
    LoadDistroList(Vec<Distro>),
    LoadBlockDeviceList(Vec<BlockDevice>),
    Err(Arc<anyhow::Error>),
    PickDistro(usize),
    OpenTargetPicker,
    TriggerDirectoryPicker,
    PickIsoDirectory(PathBuf),
    SetIsoFileName(String),
    PickBlockDeviceIndex(usize),
    StartInstall,
    BackToDistro,
    Ignore,
}

enum MainPageState {
    Distro,
    Target,
}

#[derive(Clone)]
pub enum DownloadTarget {
    BlockDev(usize),
    Directory(PathBuf),
}

pub struct MainPage {
    state: MainPageState,
    distro_list: Option<Vec<Distro>>,
    block_dev_list: Option<Vec<BlockDevice>>,
    distro_index: Option<usize>,
    download_target: Option<DownloadTarget>,
    iso_file_name: String,
    show_distro_warning: bool,
}

impl MainPage {
    pub fn new() -> Self {
        let default_download_dir = default_download_dir();
        Self {
            state: MainPageState::Distro,
            distro_list: None,
            block_dev_list: None,
            distro_index: None,
            download_target: Some(DownloadTarget::Directory(default_download_dir)),
            iso_file_name: "linux".to_owned(),
            show_distro_warning: false,
        }
    }
}

impl Page for MainPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>) {
        let mut page: Option<Box<dyn Page>> = None;
        let mut task = iced::Task::none();
        if let AppMessage::Main(msg) = message {
            match msg {
                MainPageMessage::PickDistro(distro_index) => {
                    self.distro_index = Some(distro_index);
                    self.show_distro_warning = false;
                    if let Some(name) = self
                        .distro_list
                        .as_ref()
                        .and_then(|distros| distros.get(distro_index))
                        .map(|distro| distro.name.clone())
                    {
                        self.iso_file_name = sanitize_iso_name(name);
                    }
                }
                MainPageMessage::StartInstall => {
                    if let Some(distro_index) = self.distro_index
                        && let Some(download_target) = self.download_target.as_ref()
                        && let Some(distro_list) = self.distro_list.as_ref()
                        && let Some(distro) = distro_list.get(distro_index).cloned()
                    {
                        let iso_path = match download_target {
                            DownloadTarget::BlockDev(i) => self
                                .block_dev_list
                                .as_ref()
                                .and_then(|block_dev_list| block_dev_list.get(*i))
                                .map(|dev| dev.path.clone()),
                            DownloadTarget::Directory(path_buf) => {
                                let trimmed = self.iso_file_name.trim();
                                let name = if trimmed.is_empty() { "download" } else { trimmed };
                                let filename = name
                                    .strip_suffix(".iso")
                                    .or_else(|| name.strip_suffix(".ISO"))
                                    .unwrap_or(name)
                                    .to_owned();
                                Some(path_buf.join(format!("{filename}.iso")))
                            }
                        };

                        if let Some(iso_path) = iso_path {
                            let install_settings = InstallSettings::new(distro, iso_path);
                            page = Some(Box::new(download_page::DownloadPage::new(
                                install_settings,
                            )));
                        }
                    }
                }
                MainPageMessage::BackToDistro => self.state = MainPageState::Distro,
                MainPageMessage::TriggerDirectoryPicker => {
                    task = open_folder(default_download_dir());
                }
                MainPageMessage::PickIsoDirectory(path_buf) => {
                    self.download_target = Some(DownloadTarget::Directory(path_buf));
                }
                MainPageMessage::PickBlockDeviceIndex(i) => {
                    self.download_target = Some(DownloadTarget::BlockDev(i))
                }
                MainPageMessage::SetIsoFileName(name) => {
                    self.iso_file_name = sanitize_iso_name(name)
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
                    if self.distro_index.is_some() {
                        self.state = MainPageState::Target;
                        self.show_distro_warning = false;
                    } else {
                        self.show_distro_warning = true;
                        task = Task::batch([task, show_distro_warning_dialog()]);
                    }
                }
                MainPageMessage::LoadBlockDeviceList(l) => self.block_dev_list = Some(l),
            }
        }
        if self.distro_list.is_none() {
            task = Task::batch([task, get_distro_list()]);
        }
        (page, task)
    }
    fn view(&self) -> iced::Element<'_, AppMessage> {
        let e = match self.state {
            MainPageState::Distro => self.distro_picker_view(),
            MainPageState::Target => self.target_picker_view(),
        };
        container(e)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .max_width(760)
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
        let mut content = column![
            text("Choose a distro").size(24),
            scrollable(distro_list)
                .height(Length::FillPortion(1))
                .width(Length::Fill),
            button("Next").on_press(AppMessage::Main(MainPageMessage::OpenTargetPicker))
        ];

        if self.show_distro_warning {
            content = content.push(row![text("⚠"), text("Please choose a distro")].spacing(8));
        }

        content.spacing(16).padding(8).width(Length::Fill)
    }
    fn target_picker_view(&self) -> iced::widget::Column<'_, AppMessage> {
        column![
            self.file_path_view(),
            self.block_dev_view(),
            row![
                button("Back").on_press(AppMessage::Main(MainPageMessage::BackToDistro)),
                button("Begin Download").on_press_maybe(if self.download_target.is_some() {
                    Some(AppMessage::Main(MainPageMessage::StartInstall))
                } else {
                    None
                })
            ]
            .spacing(12)
        ]
        .spacing(32)
        .padding(8)
    }
    fn file_path_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let mut col = column![text("Download to a folder").size(24),];
        if let Some(DownloadTarget::Directory(path)) = &self.download_target {
            col = col.push(text(format!("{}", path.display())));
        } else {
            col = col.push(text("No download folder selected"))
        }
        col = col.push(
            text_input("ISO name", &self.iso_file_name)
                .on_input(|name| AppMessage::Main(MainPageMessage::SetIsoFileName(name))),
        );
        col = col.push(
            row![
                button("Choose folder")
                    .on_press(AppMessage::Main(MainPageMessage::TriggerDirectoryPicker)),
                text("(optional)")
            ]
            .spacing(8)
            .align_y(iced::alignment::Vertical::Center),
        );
        container(col.spacing(16)).width(Length::Fill)
    }
    fn block_dev_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let selected_i = if let Some(DownloadTarget::BlockDev(n)) = self.download_target {
            Some(n)
        } else {
            None
        };
        let mut list = column![].spacing(16);
        if let Some(devs) = &self.block_dev_list {
            if devs.is_empty() {
                list = list.push(text("No removable device found"));
            }
            for (cur_i, dev) in devs.iter().enumerate() {
                let label = format!("{} ({})", dev.name, dev.size);
                list = list.push(radio(label, cur_i, selected_i, |_| {
                    AppMessage::Main(MainPageMessage::PickBlockDeviceIndex(cur_i))
                }));
            }
        } else {
            list = list.push(text("No removable device found"));
        }
        container(
            column![
                row![text("Flash to a disk").size(24), text("(optional)")]
                    .spacing(8)
                    .align_y(iced::alignment::Vertical::Center),
                scrollable(list)
                    .height(Length::FillPortion(1))
                    .width(Length::Fill),
            ]
            .spacing(16),
        )
        .width(Length::Fill)
    }
}

fn open_folder(default_dir: PathBuf) -> Task<AppMessage> {
    Task::future(
        rfd::AsyncFileDialog::new()
            .set_directory(default_dir)
            .pick_folder(),
    )
    .then(|handle| match handle {
        Some(file_handle) => Task::done(AppMessage::Main(MainPageMessage::PickIsoDirectory(
            file_handle.into(),
        ))),
        None => Task::done(AppMessage::Main(MainPageMessage::Ignore)),
    })
}

fn show_distro_warning_dialog() -> Task<AppMessage> {
    Task::future(
        rfd::AsyncMessageDialog::new()
            .set_title("Missing distro")
            .set_description("Please choose a distro")
            .set_level(rfd::MessageLevel::Warning)
            .show(),
    )
    .then(|_| Task::done(AppMessage::Main(MainPageMessage::Ignore)))
}

fn sanitize_iso_name(name: String) -> String {
    name.chars().filter(|c| !c.is_whitespace()).collect()
}

fn default_download_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|home| home.join("Downloads"))
        .unwrap_or_else(|| PathBuf::from("Downloads"))
}
fn get_distro_list() -> Task<AppMessage> {
    Task::future(Distro::get_all()).then(|handle| match handle {
        Ok(distros) => Task::done(AppMessage::Main(MainPageMessage::LoadDistroList(distros))),
        Err(e) => Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(e)))),
    })
}

fn get_block_dev_list() -> Task<AppMessage> {
    Task::future(disk::get_external_disks()).then(|handle| match handle {
        Ok(list) => Task::done(AppMessage::Main(MainPageMessage::LoadBlockDeviceList(list))),
        Err(e) => Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(e)))),
    })
}
