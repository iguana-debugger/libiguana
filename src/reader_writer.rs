use std::{
    io::{Read, Write},
    process::Child,
};

use crate::LibiguanaError;

/// Some functions to help with talking to Jimulator. These are in a separate struct because uniffi
/// understandably hates &mut stuff in arguments
pub struct ReaderWriter {
    // yes this struct holds nothing lol
}

impl ReaderWriter {
    /// Reads from the given process using read_exact.
    pub fn read_exact(buf: &mut [u8], process: &mut Child) -> Result<(), LibiguanaError> {
        process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_exact(buf)?;

        Ok(())
    }

    /// Reads from the given process using read_until_end.
    pub fn read_to_end(process: &mut Child) -> Result<Vec<u8>, LibiguanaError> {
        let mut buf = Vec::new();

        process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_to_end(&mut buf)?;

        Ok(buf)
    }

    /// Writes the given byte array to the given process.
    pub fn write(payload: &[u8], process: &mut Child) -> Result<(), LibiguanaError> {
        process
            .stdin
            .as_ref()
            .ok_or(LibiguanaError::NoStdin)?
            .write_all(payload)?;

        Ok(())
    }
}
