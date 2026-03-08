/* https://gist.github.com/mikroskeem/2a7e7c84f17b5fc49ca3a123dd3cb31a */
use anyhow::Result;
use nix::{cmsg_space, sys::socket::MsgFlags};
use std::{
    fs::File,
    io::{Error, IoSliceMut, Write},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
        unix::net::UnixStream,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use libc::pipe;
use passfd::FdPassingExt;
use security_framework::authorization::{Authorization, AuthorizationItemSetBuilder, Flags};
use security_framework_sys::authorization::{
    AuthorizationCreate, AuthorizationExternalForm, AuthorizationFree, AuthorizationItem,
    AuthorizationMakeExternalForm, AuthorizationRef, AuthorizationRights,
    kAuthorizationExternalFormLength, kAuthorizationFlagExtendRights,
    kAuthorizationFlagInteractionAllowed, kAuthorizationFlagPreAuthorize,
};

#[derive(Clone, Copy)]
pub enum OpenOption {
    Read,
    ReadWrite,
    ReadWriteAppend,
    ReadWriteCreate(u32),
}

// From Mark on the discord
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

    println!("1");
    let (stdin, send_fd) = unsafe {
        let mut fds = [0_i32; 2];
        if pipe(fds.as_mut_ptr()) < 0 {
            return Err(std::io::Error::last_os_error());
        }

        (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1]))
    };
    let (stdout, recv_fd) = UnixStream::pair()?;

    println!("a");
    // Spawn authopen
    let mut child = Command::new("/usr/libexec/authopen")
        .stdin(stdin)
        .stdout(OwnedFd::from(stdout))
        .arg("-extauth")
        .arg("-stdoutpipe")
        .args(&flags)
        .arg(path.as_ref())
        .spawn()?;

    println!("b");
    let (auth_ref, file) = unsafe {
        (create_authorization(send_fd, path.as_ref(), openoption)?, {
            println!("9");
            File::from_raw_fd(recv_fd.recv_fd()?)
        })
    };
    println!("c");

    let result = child.wait()?;
    if !result.success() {
        return Err(Error::other("authopen failed"));
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
                return Err(Error::other(format!(
                    "AuthorizationMakeExternalForm failed: {}",
                    ret
                )));
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
    let mut pipe = unsafe { File::from_raw_fd(pipe.as_raw_fd()) };

    let mode = match openoption {
        OpenOption::Read => "readonly",
        OpenOption::ReadWriteAppend | OpenOption::ReadWrite => "readwrite",
        OpenOption::ReadWriteCreate(_) => "readwritecreate",
    };
    let right = format!("sys.openfile.{}.{}", mode, path.as_ref().to_string_lossy());

    println!("creating auth item");
    let mut item = AuthorizationItem {
        name: right.as_bytes().as_ptr() as *const i8,
        value: std::ptr::null_mut(),
        valueLength: 0,
        flags: 0,
    };
    println!("creating auth rights");
    let rights = AuthorizationRights {
        count: 1,
        items: &mut item as *mut AuthorizationItem,
    };
    let flags = kAuthorizationFlagExtendRights
        | kAuthorizationFlagInteractionAllowed
        | kAuthorizationFlagPreAuthorize;

    println!("creating auth ref");
    let mut auth_ref = AuthRef::new();
    println!("creating auth");
    let ret =
        unsafe { AuthorizationCreate(&rights, std::ptr::null(), flags, auth_ref.as_mut_ptr()) };
    if ret < 0 {
        return Err(Error::other(format!("AuthorizationCreate failed: {}", ret)));
    }

    println!("to external form");
    let external_form = auth_ref.to_external_form()?;
    println!("transumuting");
    let bytes: [u8; kAuthorizationExternalFormLength] =
        unsafe { std::mem::transmute(external_form.bytes) };
    println!("writing to pipe");
    pipe.write_all(&bytes)?;

    Ok(auth_ref)
}

// From https://github.com/beagleboard/bb-imager-rs/blob/8334c3828f39eb08ab4f1212fd4623cc30117d1e/bb-flasher-sd/src/pal/macos.rs#L68
pub async fn open_auth(dst: &Path) -> Result<File> {
    fn inner(dst: PathBuf) -> anyhow::Result<File> {
        use nix::cmsg_space;
        use nix::sys::socket::{ControlMessageOwned, MsgFlags};
        use security_framework::authorization::{
            Authorization, AuthorizationItemSetBuilder, Flags,
        };
        use std::{
            io::{IoSliceMut, Write},
            os::{
                fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
                unix::net::UnixStream,
            },
            process::{Command, Stdio},
        };

        let rights = AuthorizationItemSetBuilder::new()
            .add_right(format!("sys.openfile.readwrite.{}", dst.to_str().unwrap()))
            .expect("Failed to create right")
            .build();

        let auth = Authorization::new(
            Some(rights),
            None,
            Flags::INTERACTION_ALLOWED | Flags::EXTEND_RIGHTS | Flags::PREAUTHORIZE,
        )
        .expect("Failed to create authorization");

        let form = auth
            .make_external_form()
            .expect("Failed to make external form");
        let (pipe0, pipe1) = UnixStream::pair().expect("Failed to create socket");

        let mut cmd = Command::new("/usr/libexec/authopen")
            .args(["-stdoutpipe", "-extauth", "-o", "2", dst.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(OwnedFd::from(pipe1))
            .spawn()?;

        // Send authorization form
        let mut stdin = cmd.stdin.take().expect("Missing stdin");
        let form_bytes: Vec<u8> = form.bytes.into_iter().map(|x| x as u8).collect();
        stdin
            .write_all(&form_bytes)
            .expect("Failed to write to stdin");
        drop(stdin);

        const IOV_BUF_SIZE: usize =
            unsafe { nix::libc::CMSG_SPACE(std::mem::size_of::<std::ffi::c_int>() as u32) }
                as usize;
        let mut iov_buf = [0u8; IOV_BUF_SIZE];
        let mut iov = [IoSliceMut::new(&mut iov_buf)];

        let mut cmsg = cmsg_space!([RawFd; 1]);

        match nix::sys::socket::recvmsg::<()>(
            pipe0.as_raw_fd(),
            &mut iov,
            Some(&mut cmsg),
            MsgFlags::empty(),
        ) {
            Ok(result) => {
                println!("Result: {:#?}", result);

                for msg in result.cmsgs().expect("Unexpected error") {
                    if let ControlMessageOwned::ScmRights(scm_rights) = msg
                        && let Some(fd) = scm_rights.into_iter().next()
                    {
                        println!("receive file descriptor");
                        return Ok(unsafe { File::from_raw_fd(fd) });
                    }
                }
            }
            Err(e) => {
                println!("Macos Error: {}", e);
            }
        }

        let _ = cmd.wait();

        Err(anyhow::anyhow!("Authopen failed to open the SD Card"))
    }

    let p = dst.to_owned();
    // TODO: Make this into a real async function
    let f = tokio::task::spawn_blocking(move || inner(p))
        .await
        .unwrap()?;

    Ok(f)
}
