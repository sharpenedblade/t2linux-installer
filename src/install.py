# tmutil deletelocalsnapshots /
# sudo diskutil apfs resizeContainer disk0s2 MACOSPARTITIONSIZEg
import os
import partitioning
from partitioning import Disk, Partition

def ask_user_for_disk() -> Disk :
    """Asks user for install disk"""
    disks = partitioning.get_all_disks()
    print("Available disks for installation:")
    index = 0
    for i in disks:
        if i.name == "disk0":
            print("disk0(internal)" + " [" + str(index) + "]")
        else:
            print(i.name + "(external)" + " [" + str(index) + "]")
        index += 1
    u = int(input("Which disk do you want to use? [0-" + str(len(disks)-1) + "]: "))
    return disks[u]

def ask_user_for_iso_disk() -> Disk :
    """Asks user for livecd disk"""
    disks = partitioning.get_all_disks()
    print("Available disks for ISO:")
    index = 0
    for i in disks:
        if i.name == "disk0":
            break
        else:
            print(i.name + "(external)" + " [" + str(index) + "]")
        index += 1
    u = int(input("Which disk do you want to use? [1-" + str(len(disks)-1) + "]: "))
    return disks[u]

def ask_user_for_distro():
    """Asks user for distro"""
    distros = ["Ubuntu", "Arch", "Fedora", "eOS"]
    index = 0
    for i in distros:
        print(i + " [" + str(index) + "]")
        index += 1
    u = int(input("Which distro? [0-" + str(len(distros)-1) + "]: "))
    return distros[u]

def shrink_macos(disk: Disk):
    """Shrink macOS partition on specified disk"""
    pass

def get_internal_disk() -> Disk:
    """Get Disk object for internal drive"""
    pass

def download_iso(distro: str):
    """Download t2 ISO for specified dostro"""
    pass

def write_iso(disk: Disk, iso):
    """Write specified iso to Disk"""
    pass

def wipe_disk(Disk):
    """Wipe specified drive"""
    pass

def main():
    distro = ask_user_for_distro()
    disk = ask_user_for_disk()
    iso_disk = ask_user_for_iso_disk()
    input("PRESS ANY KEY TO CONTINUE")
    if disk.name == "disk0":
        shrink_macos(disk)
    else:
        wipe_disk(disk)
    download_iso(distro)
    write_iso(iso_disk)

main()
