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
