/* https://gist.github.com/mikroskeem/2a7e7c84f17b5fc49ca3a123dd3cb31a */
use std::{
    fs::File,
    io::{Error, ErrorKind, Write},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::net::UnixStream,
    },
    path::Path,
    process::Command,
};

use libc::pipe;
use passfd::FdPassingExt;
use security_framework_sys::authorization::{
    kAuthorizationExternalFormLength, kAuthorizationFlagExtendRights,
    kAuthorizationFlagInteractionAllowed, kAuthorizationFlagPreAuthorize, AuthorizationCreate,
    AuthorizationExternalForm, AuthorizationFree, AuthorizationItem, AuthorizationMakeExternalForm,
    AuthorizationRef, AuthorizationRights,
};

#[derive(Clone, Copy)]
pub enum OpenOption {
    Read,
    ReadWrite,
    ReadWriteAppend,
    ReadWriteCreate(u32),
}

pub fn open_macos<P: AsRef<Path>>(path: P, openoption: OpenOption) -> Result<File, std::io::Error> {
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

    let (stdin, send_fd) = unsafe {
        let mut fds = [0_i32; 2];
        if pipe(fds.as_mut_ptr()) < 0 {
            return Err(std::io::Error::last_os_error());
        }

        (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1]))
    };
    let (stdout, recv_fd) = UnixStream::pair()?;

    // Spawn authopen
    let mut child = Command::new("/usr/libexec/authopen")
        .stdin(stdin)
        .stdout(OwnedFd::from(stdout))
        .arg("-extauth")
        .arg("-stdoutpipe")
        .args(&flags)
        .arg(path.as_ref())
        .spawn()?;

    let (auth_ref, file) = unsafe {
        (
            create_authorization(send_fd, path.as_ref(), openoption)?,
            File::from_raw_fd(recv_fd.recv_fd()?),
        )
    };

    let result = child.wait()?;
    if !result.success() {
        return Err(Error::new(ErrorKind::Other, "authopen failed"));
    }

    drop(auth_ref);

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

    pub fn to_external_form(&self) -> Result<AuthorizationExternalForm, Error> {
        let external_form: AuthorizationExternalForm = unsafe {
            let mut data = std::mem::zeroed();
            let ret = AuthorizationMakeExternalForm(self.0, &mut data);
            if ret < 0 {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("AuthorizationMakeExternalForm failed: {}", ret),
                ));
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
    pipe: OwnedFd,
    path: P,
    openoption: OpenOption,
) -> Result<AuthRef, Error> {
    let mut pipe = File::from_raw_fd(pipe.as_raw_fd());

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
    let ret = AuthorizationCreate(&rights, std::ptr::null(), flags, auth_ref.as_mut_ptr());
    if ret < 0 {
        return Err(Error::new(
            ErrorKind::Other,
            format!("AuthorizationCreate failed: {}", ret),
        ));
    }

    let external_form = auth_ref.to_external_form()?;
    let bytes: [u8; kAuthorizationExternalFormLength] = std::mem::transmute(external_form.bytes);
    pipe.write_all(&bytes)?;

    Ok(auth_ref)
}
