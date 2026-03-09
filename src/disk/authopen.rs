/* https://gist.github.com/mikroskeem/2a7e7c84f17b5fc49ca3a123dd3cb31a */
use anyhow::anyhow;
use std::{
    fs::File,
    io::Write,
    os::{
        fd::{FromRawFd, OwnedFd},
        unix::net::UnixStream,
    },
    path::Path,
    process::Command,
};

use anyhow::Context;
use libc::pipe;
use passfd::FdPassingExt;
use security_framework_sys::authorization::{
    AuthorizationCreate, AuthorizationExternalForm, AuthorizationFree, AuthorizationItem,
    AuthorizationMakeExternalForm, AuthorizationRef, AuthorizationRights,
    kAuthorizationFlagExtendRights, kAuthorizationFlagInteractionAllowed,
    kAuthorizationFlagPreAuthorize,
};

#[derive(Clone, Copy)]
pub enum OpenOption {
    Read,
    ReadWrite,
    ReadWriteAppend,
    ReadWriteCreate(u32),
}

pub fn open_macos<P: AsRef<Path>>(path: P, openoption: OpenOption) -> anyhow::Result<File> {
    let mut flags: Vec<String> = vec![];
    match openoption {
        OpenOption::Read => {}
        OpenOption::ReadWrite => {
            flags.push("-w".into());
        }
        OpenOption::ReadWriteAppend => {
            flags.push("-w".into());
            flags.push("-a".into());
        }
        OpenOption::ReadWriteCreate(mode) => {
            flags.push("-w".into());
            flags.push("-c".into());
            flags.push("-m".into());
            flags.push(format!("{:o}", mode));
        }
    }

    let (stdin_read, stdin_write) = unsafe {
        let mut fds = [0_i32; 2];
        if pipe(fds.as_mut_ptr()) < 0 {
            return Err(anyhow!(std::io::Error::last_os_error()));
        }

        (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1]))
    };
    let mut stdin_write = File::from(stdin_write);
    let (stdout, recv_fd) = UnixStream::pair().context("Failed to create unix stream pair")?;

    // Spawn authopen
    let mut child = Command::new("/usr/libexec/authopen")
        .stdin(stdin_read)
        .stdout(OwnedFd::from(stdout))
        .arg("-extauth")
        .arg("-stdoutpipe")
        .args(&flags)
        .arg(path.as_ref())
        .spawn()
        .context("Failed to spawn authopen")?;

    let auth_ref = unsafe { create_authorization(path.as_ref(), openoption) }
        .context("Failed to get authorization")?;

    stdin_write
        .write_all(
            &auth_ref
                .to_external_form()
                .context("Failed to convert authorization to external form")?
                .bytes
                .map(|i| i as u8),
        )
        .context("Failed to write authorization to authopen's stdin")?;
    let file = unsafe {
        File::from_raw_fd(
            recv_fd
                .recv_fd()
                .context("Failed to receive fd from socket")?,
        )
    };

    let result = child.wait().context("Failed to wait for anyhow to exit")?;
    if !result.success() {
        return Err(anyhow!("authopen failed with a non-zero exit code"));
    }

    Ok(file)
}

#[derive(Debug)]
struct AuthRef(AuthorizationRef);

impl AuthRef {
    pub fn new() -> Self {
        AuthRef(std::ptr::null_mut())
    }

    pub fn as_mut_ptr(&mut self) -> *mut AuthorizationRef {
        &mut self.0
    }

    pub fn to_external_form(&self) -> anyhow::Result<AuthorizationExternalForm> {
        let external_form: AuthorizationExternalForm = unsafe {
            let mut data = std::mem::zeroed();
            let ret = AuthorizationMakeExternalForm(self.0, &mut data);
            if ret < 0 {
                return Err(anyhow!("AuthorizationMakeExternalForm failed: {}", ret));
            }
            data
        };
        Ok(external_form)
    }
}

impl Drop for AuthRef {
    fn drop(&mut self) {
        unsafe {
            AuthorizationFree(self.0, 0);
        }
    }
}

unsafe fn create_authorization<P: AsRef<Path>>(
    path: P,
    openoption: OpenOption,
) -> anyhow::Result<AuthRef> {
    let mode = match openoption {
        OpenOption::Read => "readonly",
        OpenOption::ReadWriteAppend | OpenOption::ReadWrite => "readwrite",
        OpenOption::ReadWriteCreate(_) => "readwritecreate",
    };
    let right = format!("sys.openfile.{}.{}", mode, path.as_ref().to_string_lossy());

    let mut item = AuthorizationItem {
        name: right.as_bytes().as_ptr() as *const i8,
        value: std::ptr::null_mut(),
        valueLength: 0,
        flags: 0,
    };
    let rights = AuthorizationRights {
        count: 1,
        items: &mut item as *mut AuthorizationItem,
    };
    let flags = kAuthorizationFlagExtendRights
        | kAuthorizationFlagInteractionAllowed
        | kAuthorizationFlagPreAuthorize;

    let mut auth_ref = AuthRef::new();
    let ret =
        unsafe { AuthorizationCreate(&rights, std::ptr::null(), flags, auth_ref.as_mut_ptr()) };
    if ret < 0 {
        return Err(anyhow!("AuthorizationCreate failed: {}", ret));
    }
    Ok(auth_ref)
}
