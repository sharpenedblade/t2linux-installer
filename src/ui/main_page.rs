use anyhow::{Result, anyhow};
use iced::Element;
use iced::widget::{center, center_x, row, space};
use std::{path::PathBuf, sync::Arc};

use crate::disk::{self, BlockDevice};
use crate::install::DownloadTarget;
use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::{
    Length, Task,
    widget::{button, column, container, radio, scrollable, text},
};
use tokio::fs::{File, OpenOptions};

use super::finish_page::FinishPage;

#[derive(Debug, Clone)]
pub enum MainPageMessage {
    LoadDistroList(Vec<Distro>),
    LoadBlockDeviceList(Vec<BlockDevice>),
    Err(Arc<anyhow::Error>),
    PickDistro(usize),
    OpenTargetPicker,
    OpenDistroPicker,
    TriggerFilePicker,
    PickIsoFile(Arc<File>, PathBuf),
    SetBlockDeviceFile(Arc<File>),
    TriggerBlockDevicePrompt,
    PickBlockDeviceIndex(usize),
    StartInstall,
    Ignore,
}

enum MainPageState {
    Distro,
    Target,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum UIDownloadTarget {
    BlockDev(usize),
    File(PathBuf),
}

pub struct MainPage {
    state: MainPageState,
    distro_list: Option<Vec<Distro>>,
    block_dev_list: Option<Vec<BlockDevice>>,
    distro_index: Option<usize>,
    download_target: Option<UIDownloadTarget>,
    download_file: Option<File>,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            state: MainPageState::Distro,
            distro_list: None,
            block_dev_list: None,
            distro_index: None,
            download_target: None,
            download_file: None,
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
                        && let Some(download_target) = self.download_target.clone()
                        && let Some(distro_list) = self.distro_list.clone()
                        && let Some(block_dev_list) = self.block_dev_list.clone()
                        && self.download_file.is_some()
                    {
                        // Already checked to be Some(), and behind a &mut so immutable
                        let file = self.download_file.take().unwrap();
                        let download_target = match download_target {
                            UIDownloadTarget::BlockDev(i) => {
                                DownloadTarget::BlockDev(block_dev_list[i].clone())
                            }
                            UIDownloadTarget::File(path_buf) => DownloadTarget::File(path_buf),
                        };
                        let install_settings = InstallSettings::new(
                            distro_list.get(distro_index).unwrap().clone(),
                            download_target,
                        );
                        page = Some(Box::new(download_page::DownloadPage::new(
                            install_settings,
                            file,
                        )))
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
                MainPageMessage::PickIsoFile(file, path_buf) => {
                    let file = Arc::try_unwrap(file).unwrap();
                    self.download_file = Some(file);
                    self.download_target = Some(UIDownloadTarget::File(path_buf));
                }
                MainPageMessage::PickBlockDeviceIndex(i) => {
                    self.download_target = Some(UIDownloadTarget::BlockDev(i))
                }
                MainPageMessage::SetBlockDeviceFile(file) => {
                    let file = Arc::try_unwrap(file).unwrap();
                    self.download_file = Some(file);
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
                MainPageMessage::LoadBlockDeviceList(l) => self.block_dev_list = Some(l),
                MainPageMessage::TriggerBlockDevicePrompt => {
                    if let Some(block_devices) = &self.block_dev_list
                        && let Some(UIDownloadTarget::BlockDev(i)) = self.download_target
                    {
                        let block_device = block_devices[i].clone();
                        task = get_block_dev_fd(block_device);
                    }
                }
                MainPageMessage::OpenDistroPicker => {
                    self.state = MainPageState::Distro;
                }
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
        e.into()
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
            center_x(
                column![
                    text("Choose a distro").size(24).height(Length::Shrink),
                    scrollable(distro_list).height(Length::Fill)
                ]
                .spacing(16)
            )
            .height(Length::Fill),
            column![
                space::vertical(),
                row![
                    space::horizontal(),
                    button("Next").on_press(AppMessage::Main(MainPageMessage::OpenTargetPicker))
                ]
            ]
            .height(Length::Fixed(32.0))
        ]
        // .spacing(16)
        .padding(16)
    }
    fn target_picker_view(&self) -> iced::widget::Column<'_, AppMessage> {
        column![
            center_x(column![self.file_path_view(), self.block_dev_view()].spacing(32)),
            space::vertical(),
            row![
                button("Back").on_press(AppMessage::Main(MainPageMessage::OpenDistroPicker)),
                space::horizontal(),
                button("Begin Download").on_press_maybe(if self.download_target.is_some() {
                    Some(AppMessage::Main(MainPageMessage::StartInstall))
                } else {
                    None
                })
            ]
            .height(Length::Fixed(32.0)),
        ]
        .padding(16)
    }
    fn file_path_view(&self) -> Element<'_, AppMessage> {
        let mut col = column![text("Download to a file").size(24),];
        if let Some(UIDownloadTarget::File(path)) = &self.download_target {
            col = col.push(text(format!("{}", path.display())));
        } else {
            col = col.push(text("No download path selected"))
        }
        col = col.push(
            button("Choose file path ")
                .on_press(AppMessage::Main(MainPageMessage::TriggerFilePicker)),
        );
        col.spacing(16).into()
    }
    fn block_dev_view(&self) -> Element<'_, AppMessage> {
        let selected_i = if let Some(UIDownloadTarget::BlockDev(n)) = self.download_target {
            Some(n)
        } else {
            None
        };
        let mut list = column![].spacing(16);
        if let Some(devs) = &self.block_dev_list {
            for (cur_i, dev) in devs.iter().enumerate() {
                let label = format!("{} ({})", dev.name, dev.size);
                list = list.push(radio(label, cur_i, selected_i, |_| {
                    AppMessage::Main(MainPageMessage::PickBlockDeviceIndex(cur_i))
                }));
            }
        }
        column![
            text("Flash to a disk").size(24),
            scrollable(list).height(Length::Shrink),
            button("Open Device").on_press_maybe(match self.download_target {
                Some(UIDownloadTarget::BlockDev(_)) =>
                    Some(AppMessage::Main(MainPageMessage::TriggerBlockDevicePrompt)),
                _ => None,
            })
        ]
        .spacing(16)
        .into()
    }
}

fn open_file(name: String) -> Task<AppMessage> {
    Task::future(async {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("ISO files", &["iso"])
            .set_file_name(name)
            .save_file()
            .await
        {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(handle.path())
                .await?;
            Ok(Some((
                Arc::new(file.into_std().await.into()),
                handle.path().to_owned(),
            )))
        } else {
            Ok(None)
        }
    })
    .then(|res: Result<Option<(Arc<File>, PathBuf)>>| {
        let o = match res {
            Ok(o) => o,
            Err(e) => {
                return Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(anyhow!(e)))));
            }
        };
        match o {
            Some((file, path)) => {
                Task::done(AppMessage::Main(MainPageMessage::PickIsoFile(file, path)))
            }
            None => Task::done(AppMessage::Main(MainPageMessage::Ignore)),
        }
    })
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

fn get_block_dev_fd(b: BlockDevice) -> Task<AppMessage> {
    Task::future(disk::get_fd_for_disk(b)).then(|handle| match handle {
        Ok(file) => Task::done(AppMessage::Main(MainPageMessage::SetBlockDeviceFile(
            Arc::new(file),
        ))),
        Err(e) => Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(e)))),
    })
}
