use passfd::FdPassingExt;
use sendfd::RecvWithFd;
use std::os::unix::net::UnixStream;
use std::{fs::File, os::fd::OwnedFd, os::unix::io::FromRawFd, process::Command};

pub fn open_restricted_file(path: &str) -> std::fs::File {
    let (cmd_sock, out_sock) = UnixStream::pair().unwrap();

    let args = ["-stdoutpipe", "-c", "-w", path];

    let _cmd = Command::new("/usr/libexec/authopen")
        .args(args)
        .stdout(OwnedFd::from(cmd_sock))
        .spawn()
        .unwrap();

    let mut by = [0u8; 10];
    let mut fds = [std::os::fd::RawFd::default(); 8];
    out_sock.recv_with_fd(&mut by, &mut fds).unwrap();
    let fd = fds.first().unwrap().clone();

    unsafe { File::from_raw_fd(fd) }
}
