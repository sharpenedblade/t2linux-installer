use crate::distro::Distro;
use crate::error::Error;
use futures::{Stream, StreamExt};
use std::fs;

#[derive(Debug, Clone, Eq, PartialEq)]
enum InstallStep {
    Start,
    DownloadIso,
    Finished,
}

#[derive(Debug)]
pub enum InstallProgress {
    DownloadingIso,
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
}

impl InstallSettings {
    pub fn new(distro: Distro) -> Self {
        Self { distro }
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
                        Some((InstallProgress::DownloadingIso, state))
                    }
                    InstallStep::DownloadIso => {
                        let Ok(iso) = state.settings.distro.download_iso().await else {
                            state.step = InstallStep::Finished;
                            return Some((InstallProgress::Failed(Error::IsoDownload), state));
                        };
                        state.iso_file = Some(iso);
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
