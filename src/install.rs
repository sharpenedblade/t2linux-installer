use crate::distro::Distro;
use crate::error::Error;
use anyhow::Result;
use futures::{Stream, StreamExt};

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
    async fn flash_iso(&self) -> Result<()> {
        Ok(())
    }
    pub fn install(&self) -> impl Stream<Item = InstallProgress> {
        let settings = self.clone();
        futures::stream::unfold(
            Installer {
                step: InstallStep::Start,
                settings,
            },
            |mut state| async move {
                match state.step {
                    InstallStep::Start => {
                        state.step = InstallStep::DownloadIso;
                        Some((InstallProgress::Started, state))
                    }
                    InstallStep::DownloadIso => {
                        let Ok(_iso) = state.settings.distro.download_iso().await else {
                            state.step = InstallStep::Finished;
                            return Some((InstallProgress::Failed(Error::IsoDownload), state));
                        };
                        state.step = InstallStep::FlashIso;
                        Some((InstallProgress::DownloadedIso, state))
                    }
                    InstallStep::FlashIso => {
                        let Ok(_) = state.settings.flash_iso().await else {
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
