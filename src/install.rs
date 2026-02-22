use crate::distro::Distro;
use crate::error::Error;
use futures::{Stream, StreamExt};
use std::fs;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Eq, PartialEq)]
enum InstallStep {
    Start,
    DownloadIso,
    Finished,
}

#[derive(Debug)]
pub enum InstallProgress {
    IsoDownloadStart,
    IsoDownloadProgress(f64),
    Finished,
    Failed(Error),
}

#[derive(Debug)]
struct Installer {
    step: InstallStep,
    settings: DownloadSettings,
    iso_file: Option<fs::File>,
    ct: CancellationToken,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DownloadSettings {
    distro: Distro,
}

impl DownloadSettings {
    pub fn new(distro: Distro) -> Self {
        Self { distro }
    }
    pub fn install(&self, ct: CancellationToken) -> impl Stream<Item = InstallProgress> {
        let settings = self.clone();
        futures::stream::unfold(
            Installer {
                step: InstallStep::Start,
                settings,
                iso_file: None,
                ct,
            },
            |mut state| async move {
                match state.step {
                    InstallStep::Start => {
                        state.step = InstallStep::DownloadIso;
                        Some((InstallProgress::IsoDownloadStart, state))
                    }
                    InstallStep::DownloadIso => {
                        let Ok(iso) = state.settings.distro.download_iso(state.ct.clone()).await
                        else {
                            state.step = InstallStep::Finished;
                            return Some((
                                InstallProgress::Failed(if state.ct.clone().is_cancelled() {
                                    Error::Cancelled
                                } else {
                                    Error::IsoDownload
                                }),
                                state,
                            ));
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
