use crate::disk::BlockDevice;
use anyhow::Result;
use humansize::{DECIMAL, format_size};

pub fn get_external_disks() -> Result<Vec<BlockDevice>> {
    let all_devices = blockdev::get_devices()?;
    Ok(all_devices
        .into_iter()
        // .filter(|d| d.is_disk() && !d.is_mounted())
        .map(|d| BlockDevice {
            os_identifier: d.name.clone(),
            name: d.name,
            size: format_size(d.size, DECIMAL),
        })
        .collect())
}
