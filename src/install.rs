use crate::distro::Distro;
use futures::{Stream, StreamExt};

#[derive(Debug, Clone, Eq, PartialEq)]
enum InstallStep {
    Start,
    DownloadIso,
    Finished,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InstallProgress {
    Started,
    Finished,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Installer {
    step: InstallStep,
    settings: InstallSettings,
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
            },
            |state| async move {
                let mut next_state: Installer = state.clone();
                match state.step {
                    InstallStep::Start => {
                        next_state.step = InstallStep::DownloadIso;
                        Some((InstallProgress::Started, next_state))
                    }
                    InstallStep::DownloadIso => {
                        state.settings.distro.download_iso().await.unwrap();
                        next_state.step = InstallStep::Finished;
                        Some((InstallProgress::Finished, next_state))
                    }
                    InstallStep::Finished => None,
                }
            },
        )
        .boxed()
    }
}
