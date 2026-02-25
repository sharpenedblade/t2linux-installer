use anyhow::Result;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
pub mod diskutil;

#[cfg(target_os = "linux")]
mod lsblk;

#[derive(Debug, Clone)]
pub struct BlockDevice {
    pub name: String,
    pub size: String,
    pub path: PathBuf,
}

#[cfg(target_os = "macos")]
pub async fn get_external_disks() -> Result<Vec<BlockDevice>> {
    tokio::task::spawn_blocking(diskutil::get_external_disks)
        .await
        .unwrap()
}

#[cfg(target_os = "linux")]
pub async fn get_external_disks() -> Result<Vec<BlockDevice>> {
    tokio::task::spawn_blocking(lsblk::get_external_disks)
        .await
        .unwrap()
}
