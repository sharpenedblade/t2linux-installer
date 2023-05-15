from ctypes import sizeof
import plistlib
import subprocess
from dataclasses import dataclass
from typing import List

@dataclass
class Partition:
    size: int
    guid: str
    name: str

    def __repr__(self):
        return self.name + ", guid=" + self.guid + ", size=" + str(self.size)

@dataclass
class Disk:
    name: str
    size: int
    partitions: List[Partition]

    def __repr__(self):
        s = self.name + ", " + "size=" + str(self.size) + "\n"
        for i in self.partitions:
            s = s + "\t" + str(i) + "\n"
        return s

def new_disk_from_name(name: str):
    diskutil_out = subprocess.run(['diskutil', 'list', '-plist', name], stdout=subprocess.PIPE).stdout
    disk_data = plistlib.loads(diskutil_out)
    partitions = []
    for p in disk_data["AllDisksAndPartitions"][0]["Partitions"]:
        part = Partition(
                name = p["DeviceIdentifier"],
                guid = p["DiskUUID"],
                size = p["Size"],
                )
        partitions.append(part)
    return Disk(
            name=name,
            size=disk_data["AllDisksAndPartitions"][0]["Size"],
            partitions=partitions
            )

def get_all_disks():
    diskutil_out = subprocess.run(['diskutil', 'list', '-plist'], stdout=subprocess.PIPE).stdout
    disk_data = plistlib.loads(diskutil_out)
    l = disk_data["WholeDisks"]
    d = []
    for i in l:
        d.append(new_disk_from_name(i))
    return d
