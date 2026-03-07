use crate::disk::BlockDevice;
use crate::distro::Distro;
use crate::error::Error;
use futures::Stream;
use iced::{stream::channel, task::Sipper};
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
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
    file: Arc<File>,
}

#[derive(Debug, Clone, Hash)]
pub struct InstallSettings {
    distro: Distro,
    download_target: DownloadTarget,
}

#[derive(Debug, Clone, Hash)]
pub enum DownloadTarget {
    BlockDev(BlockDevice),
    File(PathBuf),
}

impl Display for DownloadTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadTarget::BlockDev(block_device) => {
                write!(f, "Block Device {}", block_device.name)
            }
            DownloadTarget::File(path_buf) => write!(f, "{}", path_buf.display()),
        }
    }
}

impl InstallSettings {
    pub fn new(distro: Distro, download_target: DownloadTarget) -> Self {
        Self {
            distro,
            download_target,
        }
    }

    pub fn is_block_device_target(&self) -> bool {
        matches!(self.download_target, DownloadTarget::BlockDev(_))
    }

    pub fn install(
        &self,
        file: Arc<File>,
        ct: CancellationToken,
    ) -> impl Stream<Item = InstallProgress> + use<> {
        let settings = self.clone();
        let state = Installer { settings, file, ct };
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
                    .download_iso(state.file, state.ct.clone())
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
                                    Error::IsoDownload(e.context(format!(
                                        "Failed to download ISO to {}",
                                        state.settings.download_target
                                    )))
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
