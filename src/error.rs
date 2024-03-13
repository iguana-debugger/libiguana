use std::{array::TryFromSliceError, io, num::TryFromIntError, str, string::FromUtf8Error};
use thiserror::Error;

#[derive(Debug, Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum LibiguanaError {
    #[error("jimulator process has no stdin")]
    NoStdin,

    #[error("jimulator process has no stdout")]
    NoStdout,

    #[error("An IO error occured when reading/writing to jimulator: {0:?}")]
    IO(#[from] io::Error),

    #[error("A string that was not valid UTF-8 was returned")]
    Utf8Error(#[from] str::Utf8Error),

    // TODO: Give error information on parse errors
    #[error("The given kmd file failed to parse")]
    ParseError,

    #[error("An integer overflow occured")]
    IntegerOverflow(#[from] TryFromIntError),

    #[error("Converting from a slice failed (this should never happen)")]
    TryFromSliceError(#[from] TryFromSliceError),

    #[error("Jimulator returned an invalid status {0:#03x}")]
    InvalidStatus(u8),

    #[error("The register buffer returned an invalid size {0} (should never happen!)")]
    InvalidRegisterBufferLength(usize),

    #[error("The aasm binary was not found.")]
    AasmDoesNotExist,

    #[error("A string that was not valid UTF-8 was returned")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("The mnemonics file was not found.")]
    MnemonicsDoesNotExist,
}
