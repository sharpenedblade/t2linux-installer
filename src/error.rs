use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to write file")]
    FileWrite(#[from] std::io::Error),
    #[error("failed to download ISO")]
    IsoDownload,
    #[error("failed to flash ISO to disk")]
    IsoFlash,
    #[error("failed to shrink MacOS partition")]
    MacosResize,
}
