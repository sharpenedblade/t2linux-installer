use crate::distro::Distro;
use crate::error::Error;
use futures::Stream;
use iced::{stream::channel, task::Sipper};
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub enum InstallProgress {
    /// Part
    IsoDownloadStart(usize),
    /// Part, Progress
    IsoDownloadProgress(usize, f64),
    Finished,
    Failed(Error),
}

#[derive(Debug)]
struct Installer {
    settings: InstallSettings,
    ct: CancellationToken,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InstallSettings {
    distro: Distro,
    iso_path: PathBuf,
}

impl InstallSettings {
    pub fn new(distro: Distro, iso_path: PathBuf) -> Self {
        Self { distro, iso_path }
    }
    pub fn install(&self, ct: CancellationToken) -> impl Stream<Item = InstallProgress> + use<> {
        let settings = self.clone();
        let state = Installer { settings, ct };
        channel(
            10,
            async move |mut sender: futures_channel::mpsc::Sender<InstallProgress>| {
                sender
                    .try_send(InstallProgress::IsoDownloadStart(
                        state.settings.distro.iso.len(),
                    ))
                    .unwrap();
                let mut download = state
                    .settings
                    .distro
                    .download_iso(state.settings.iso_path.clone(), state.ct.clone())
                    .pin();
                while let Some((part, progress)) = download.sip().await {
                    sender
                        .try_send(InstallProgress::IsoDownloadProgress(part, progress))
                        .unwrap();
                }
                match download.await {
                    Ok(iso) => iso,
                    Err(e) => {
                        sender
                            .try_send(InstallProgress::Failed(
                                if state.ct.clone().is_cancelled() {
                                    Error::Cancelled
                                } else {
                                    Error::IsoDownload(e)
                                },
                            ))
                            .unwrap();
                        return;
                    }
                };
                sender.try_send(InstallProgress::Finished).unwrap();
            },
        )
    }
}
