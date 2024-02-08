use std::{io, str};

#[derive(Debug)]
pub enum LibiguanaError {
    NoStdin,
    NoStdout,
    IO(io::Error),
    Utf8Error(str::Utf8Error),
}
