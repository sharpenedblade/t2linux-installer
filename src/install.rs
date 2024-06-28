use crate::diskutil;
use crate::distro::Distro;
use crate::error::Error;
use anyhow::Result;
use futures::{Stream, StreamExt};
use std::fs;
use std::io;

#[derive(Debug, Clone, Eq, PartialEq)]
enum InstallStep {
    Start,
    DownloadIso,
    FlashIso,
    ResizeMacos,
    Finished,
}

#[derive(Debug)]
pub enum InstallProgress {
    Started,
    DownloadedIso,
    ResizingMacos,
    Finished,
    Failed(Error),
}

#[derive(Debug)]
struct Installer {
    step: InstallStep,
    settings: InstallSettings,
    iso_file: Option<fs::File>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InstallSettings {
    distro: Distro,
    flash_disk: String,
    macos_size: Option<u64>,
}

impl InstallSettings {
    pub fn new(distro: Distro, flash_disk: String, macos_size: Option<u64>) -> Self {
        Self {
            distro,
            flash_disk,
            macos_size,
        }
    }
    async fn flash_iso(&self, iso_file: &mut fs::File) -> Result<()> {
        let mut target_disk = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(format!("/dev/{}", &self.flash_disk))?;
        io::copy(iso_file, &mut target_disk)?;
        Ok(())
    }
    pub fn install(&self) -> impl Stream<Item = InstallProgress> {
        let settings = self.clone();
        futures::stream::unfold(
            Installer {
                step: InstallStep::Start,
                settings,
                iso_file: None,
            },
            |mut state| async move {
                match state.step {
                    InstallStep::Start => {
                        state.step = InstallStep::DownloadIso;
                        Some((InstallProgress::Started, state))
                    }
                    InstallStep::DownloadIso => {
                        let Ok(iso) = state.settings.distro.download_iso().await else {
                            state.step = InstallStep::Finished;
                            return Some((InstallProgress::Failed(Error::IsoDownload), state));
                        };
                        state.iso_file = Some(iso);
                        state.step = InstallStep::FlashIso;
                        Some((InstallProgress::DownloadedIso, state))
                    }
                    InstallStep::FlashIso => {
                        let Ok(_) = state
                            .settings
                            .flash_iso(state.iso_file.as_mut().unwrap())
                            .await
                        else {
                            state.step = InstallStep::Finished;
                            return Some((InstallProgress::Failed(Error::IsoFlash), state));
                        };
                        state.step = InstallStep::ResizeMacos;
                        Some((InstallProgress::ResizingMacos, state))
                    }
                    InstallStep::ResizeMacos => {
                        if let Some(size) = &state.settings.macos_size.clone() {
                            if let Some(disk) = diskutil::get_internal_macos_partition() {
                                let Ok(_) = diskutil::resize_apfs_volume(&disk, size.to_owned())
                                else {
                                    state.step = InstallStep::Finished;
                                    return Some((
                                        InstallProgress::Failed(Error::MacosResize),
                                        state,
                                    ));
                                };
                            } else {
                                state.step = InstallStep::Finished;
                                return Some((InstallProgress::Failed(Error::MacosResize), state));
                            };
                        };
                        state.step = InstallStep::Finished;
                        Some((InstallProgress::Finished, state))
                    }
                    InstallStep::Finished => None,
                }
            },
        )
        .boxed()
    }
}
