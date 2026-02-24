use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to write file")]
    FileWrite(#[from] std::io::Error),
    #[error(transparent)]
    IsoDownload(#[from] anyhow::Error),
    #[error("installation cancelled")]
    Cancelled,
}
