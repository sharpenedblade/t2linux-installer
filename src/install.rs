use crate::distro::Distro;
use crate::error::Error;
use futures::Stream;
use iced::{stream::channel, task::Sipper};
use std::fs;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub enum InstallProgress {
    IsoDownloadStart,
    IsoDownloadProgress(f64),
    Finished,
    Failed(Error),
}

#[derive(Debug)]
struct Installer {
    settings: InstallSettings,
    iso_file: Option<fs::File>,
    ct: CancellationToken,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InstallSettings {
    distro: Distro,
}

impl InstallSettings {
    pub fn new(distro: Distro) -> Self {
        Self { distro }
    }
    pub fn install(&self, ct: CancellationToken) -> impl Stream<Item = InstallProgress> {
        let settings = self.clone();
        let mut state = Installer {
            settings,
            iso_file: None,
            ct,
        };
        channel(
            10,
            async move |mut sender: futures_channel::mpsc::Sender<InstallProgress>| {
                sender.try_send(InstallProgress::IsoDownloadStart).unwrap();
                let mut download = state.settings.distro.download_iso(state.ct.clone()).pin();
                while let Some(progress) = download.sip().await {
                    sender
                        .try_send(InstallProgress::IsoDownloadProgress(progress))
                        .unwrap();
                }
                let Ok(iso) = download.await else {
                    sender
                        .try_send(InstallProgress::Failed(
                            if state.ct.clone().is_cancelled() {
                                Error::Cancelled
                            } else {
                                Error::IsoDownload
                            },
                        ))
                        .unwrap();
                    return;
                };
                state.iso_file = Some(iso);
                sender.try_send(InstallProgress::Finished).unwrap();
            },
        )
    }
}
