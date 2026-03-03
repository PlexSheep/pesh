use std::io;

use thiserror::Error;

pub type PeshResult<T> = std::result::Result<T, PeshError>;

#[derive(Error, Debug)]
pub enum PeshError {
    #[error("os error")]
    Os(#[from] io::Error),
}
