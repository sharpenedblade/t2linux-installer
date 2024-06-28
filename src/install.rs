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
    Finished,
}

#[derive(Debug)]
pub enum InstallProgress {
    Started,
    DownloadedIso,
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
}

impl InstallSettings {
    pub fn new(distro: Distro, flash_disk: String) -> Self {
        Self { distro, flash_disk }
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
