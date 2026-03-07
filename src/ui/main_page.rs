use anyhow::{Result, anyhow};
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
    widget::{button, column, container, radio, row, scrollable, text},
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
    TriggerFilePicker,
    PickIsoFile(Arc<File>, PathBuf),
    SetBlockDeviceFile(Arc<File>),
    TriggerBlockDevicePrompt,
    PickBlockDeviceIndex(usize),
    StartInstall,
    BackToDistro,
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
    show_distro_warning: bool,
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
                }
                MainPageMessage::StartInstall => {
                    if let Some(distro_index) = self.distro_index
                        && let Some(distro_list) = self.distro_list.clone()
                    {
                        if let Some(distro) = distro_list.get(distro_index).cloned() {
                            let default_filename = format!("{}.iso", ensure_t2_suffix(distro.name.clone()));
                            let target_and_file =
                                match (self.download_target.clone(), self.download_file.take()) {
                                    (Some(UIDownloadTarget::BlockDev(i)), Some(file)) => {
                                        self.block_dev_list.clone().and_then(|block_dev_list| {
                                            block_dev_list.get(i).cloned().map(|block_dev| {
                                                (DownloadTarget::BlockDev(block_dev), file)
                                            })
                                        })
                                    }
                                    (Some(UIDownloadTarget::File(path_buf)), Some(file)) => {
                                        Some((DownloadTarget::File(path_buf), file))
                                    }
                                    _ => match open_default_file(default_filename) {
                                        Ok((file, path)) => Some((DownloadTarget::File(path), file)),
                                        Err(e) => {
                                            task = Task::done(AppMessage::Main(
                                                MainPageMessage::Err(Arc::new(anyhow!(e))),
                                            ));
                                            None
                                        }
                                    },
                                };

                            if let Some((download_target, file)) = target_and_file {
                                let install_settings = InstallSettings::new(distro, download_target);
                                page = Some(Box::new(download_page::DownloadPage::new(
                                    install_settings,
                                    file,
                                )));
                            }
                        }
                    }
                }
                MainPageMessage::BackToDistro => self.state = MainPageState::Distro,
                MainPageMessage::TriggerFilePicker => {
                    let file_name = self
                        .distro_index
                        .and_then(|i| {
                            self.distro_list
                                .as_ref()
                                .and_then(|distros| distros.get(i))
                                .map(|distro| ensure_t2_suffix(distro.name.clone()))
                        })
                        .unwrap_or_else(|| "linux-T2".to_owned());
                    task = open_file(format!("{file_name}.iso"));
                }
                MainPageMessage::PickIsoFile(file, path_buf) => {
                    let file = Arc::try_unwrap(file).unwrap();
                    self.download_file = Some(file);
                    self.download_target = Some(UIDownloadTarget::File(path_buf));
                    task = Task::done(AppMessage::Main(MainPageMessage::StartInstall));
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
                    if self.distro_index.is_some() {
                        self.state = MainPageState::Target;
                        self.show_distro_warning = false;
                    } else {
                        self.show_distro_warning = true;
                    }
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
                .height(320)
                .width(Length::Fill),
            button("Next").on_press(AppMessage::Main(MainPageMessage::OpenTargetPicker))
        ];

        if self.show_distro_warning {
            content = content.push(
                container(row![text("⚠"), text("Please choose a distro")].spacing(8))
                    .padding(8)
                    .width(Length::Fill),
            );
        }

        content.spacing(16).padding(8).width(Length::Fill)
    }
    fn target_picker_view(&self) -> iced::widget::Column<'_, AppMessage> {
        column![
            scrollable(column![self.file_path_view(), self.block_dev_view(),].spacing(24))
                .height(Length::Fill),
            row![
                button("Back").on_press(AppMessage::Main(MainPageMessage::BackToDistro)),
                button("Begin Download").on_press(AppMessage::Main(MainPageMessage::StartInstall))
            ]
            .spacing(12)
        ]
        .spacing(20)
        .padding(8)
        .height(Length::Fill)
    }
    fn file_path_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let mut col = column![
            text("Download to a file").size(24),
            text("By default it is saved to ~/Downloads"),
        ];
        if let Some(UIDownloadTarget::File(path)) = &self.download_target {
            col = col.push(text(format!("{}", path.display())));
        }
        col = col.push(button("Save as").on_press(AppMessage::Main(
            MainPageMessage::TriggerFilePicker,
        )));
        container(col.spacing(16)).width(Length::Fill)
    }
    fn block_dev_view(&self) -> iced::widget::Container<'_, AppMessage> {
        let selected_i = if let Some(UIDownloadTarget::BlockDev(n)) = self.download_target {
            Some(n)
        } else {
            None
        };
        let mut list = column![].spacing(16);
        if let Some(devs) = &self.block_dev_list {
            if devs.is_empty() {
                list = list.push(text("No removable devices found"));
            }
            for (cur_i, dev) in devs.iter().enumerate() {
                let label = format!("{} ({})", dev.name, dev.size);
                list = list.push(radio(label, cur_i, selected_i, |_| {
                    AppMessage::Main(MainPageMessage::PickBlockDeviceIndex(cur_i))
                }));
            }
        } else {
            list = list.push(text("No removable devices found"));
        }
        container(
            column![
                text("Flash to a disk (optional)").size(24),
                list,
                button("Open Device").on_press_maybe(match self.download_target {
                    Some(UIDownloadTarget::BlockDev(_)) =>
                        Some(AppMessage::Main(MainPageMessage::TriggerBlockDevicePrompt)),
                    _ => None,
                })
            ]
            .spacing(16),
        )
        .width(350)
    }
}

fn open_file(name: String) -> Task<AppMessage> {
    Task::future(async {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("ISO files", &["iso"])
            .set_directory(default_download_dir())
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

fn ensure_t2_suffix(name: String) -> String {
    let base_name = name
        .strip_suffix(".iso")
        .or_else(|| name.strip_suffix(".ISO"))
        .unwrap_or(name.as_str())
        .trim();
    let lowered = base_name.to_ascii_lowercase();
    if lowered.ends_with("_t2") || lowered.ends_with("-t2") || lowered.ends_with(" t2") {
        let stem = base_name[..base_name.len() - 3]
            .trim_end_matches(['_', '-', ' '])
            .trim_end();
        format!("{stem}_T2")
    } else {
        format!("{base_name}_T2")
    }
}

fn open_default_file(name: String) -> Result<(File, PathBuf)> {
    let path = default_download_dir().join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    Ok((File::from_std(file), path))
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

fn get_block_dev_fd(b: BlockDevice) -> Task<AppMessage> {
    Task::future(disk::get_fd_for_disk(b)).then(|handle| match handle {
        Ok(file) => Task::done(AppMessage::Main(MainPageMessage::SetBlockDeviceFile(
            Arc::new(file),
        ))),
        Err(e) => Task::done(AppMessage::Main(MainPageMessage::Err(Arc::new(e)))),
    })
}
