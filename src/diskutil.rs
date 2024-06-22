use anyhow::Result;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DiskList {
    all_disks: Vec<String>,
    all_disks_and_partitions: Vec<Disk>,
    volumes_from_disks: Vec<String>,
    whole_disks: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Disk {
    content: String,
    device_identifier: String,
    OS_internal: bool,
    partitions: Vec<Partition>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Partition {
    content: String,
    device_identifier: String,
    disk_UUID: Uuid,
    size: u64,
    volume_name: Option<String>,
    volume_UUID: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ApfsResizeLimits {
    container_current_size: u64,
    current_size: u64,
    maximum_size: u64,
    minimum_size_no_guard: u64,
    minimum_size_preferred: u64,
    #[serde(rename = "Type")]
    partition_type: String,
}

fn diskutil_cmd(args: Vec<&str>) -> Vec<u8> {
    let cmd = std::process::Command::new("diskutil")
        .args(args)
        .output()
        .unwrap();
    cmd.stdout
}

pub fn get_external_disks() -> Vec<String> {
    let diskutil_output = diskutil_cmd(vec!["list", "-plist", "external", "physical"]);
    let all_disks: DiskList = plist::from_bytes(diskutil_output.as_ref()).unwrap();
    let mut disks: Vec<String> = vec![];
    for disk in all_disks.whole_disks {
        disks.push(disk.clone());
    }
    disks
}

pub fn get_resize_limits(disk: &str) -> (u64, u64) {
    let diskutil_output = diskutil_cmd(vec!["apfs", "resizeContainer", disk, "limits", "-plist"]);
    let limits: ApfsResizeLimits = plist::from_bytes(diskutil_output.as_ref()).unwrap();
    (limits.minimum_size_no_guard, limits.maximum_size)
}

pub fn resize_apfs_volume(disk: &str, new_size: u64) -> Result<()> {
    let (min_size, max_size) = get_resize_limits(&disk);
    if new_size < min_size || new_size > max_size {
        anyhow::bail!("New volume size outside of acceptable range");
    }
    diskutil_cmd(vec![
        "apfs",
        "resizeContainer",
        disk,
        new_size.to_string().as_ref(),
    ]);
    Ok(())
}
