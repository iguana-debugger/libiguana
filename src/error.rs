use std::{io, num::TryFromIntError, str};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibiguanaError {
    #[error("jimulator process has no stdin")]
    NoStdin,

    #[error("jimulator process has no stdout")]
    NoStdout,

    #[error("An IO error occured when reading/writing to jimulator")]
    IO(#[from] io::Error),

    #[error("jimulator returned a string that was not valid UTF-8")]
    Utf8Error(#[from] str::Utf8Error),

    // TODO: Give error information on parse errors
    #[error("The given kmd file failed to parse")]
    ParseError,

    #[error("An integer overflow occured")]
    IntegerOverflow(#[from] TryFromIntError),
}
