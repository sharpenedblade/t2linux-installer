use anyhow::Result;

#[cfg(target_os = "macos")]
pub mod authopen;
#[cfg(target_os = "macos")]
pub mod diskutil;
#[cfg(target_os = "linux")]
mod lsblk;

#[derive(Debug, Clone, Hash)]
pub struct BlockDevice {
    pub name: String,
    pub size: String,
    pub os_identifier: String,
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

#[cfg(target_os = "linux")]
pub async fn get_fd_for_disk(b: BlockDevice) -> Result<tokio::fs::File> {
    use std::{collections::HashMap, os::fd::OwnedFd};
    use udisks2::zbus::zvariant;
    let client = udisks2::Client::new().await?;
    let object = client.object(format!(
        "/org/freedesktop/UDisks2/block_devices/{}",
        b.os_identifier
    ))?;
    let block = object.block().await?;
    let fd = block
        .open_device(
            "rw",
            HashMap::from([("O_EXCL", zvariant::Value::Bool(true))]),
        )
        .await?;
    let fd: OwnedFd = fd.into();
    Ok(tokio::fs::File::from(std::fs::File::from(fd)))
}

#[cfg(target_os = "macos")]
pub async fn get_fd_for_disk(b: BlockDevice) -> Result<tokio::fs::File> {
    use std::path::PathBuf;
    let file: Result<std::fs::File> = tokio::task::spawn_blocking(|| -> Result<std::fs::File> {
        let path = PathBuf::from("/dev").join(b.os_identifier);
        let opts = authopen::OpenOption::ReadWrite;
        let file = authopen::open_macos(path, opts)?;
        Ok(file)
    })
    .await?;
    let file = tokio::fs::File::from_std(file?);
    Ok(file)
}
