use std::{
    io::{self, Read, Write},
    process::{Child, Command, Stdio},
    str,
};

mod error;
pub use self::error::LibiguanaError;

pub struct Environment {
    jimulator_process: Child,
}

impl Environment {
    pub fn new() -> Result<Self, io::Error> {
        let jimulator_process = Command::new("jimulator")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self { jimulator_process })
    }

    pub fn ping(&mut self) -> Result<String, LibiguanaError> {
        self.jimulator_process
            .stdin
            .as_ref()
            .ok_or(LibiguanaError::NoStdin)?
            .write(&[0b0000_0001])
            .map_err(|e| LibiguanaError::IO(e))?;

        let mut buf = [0; 4];

        self.jimulator_process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_exact(&mut buf)
            .map_err(|e| LibiguanaError::IO(e))?;

        let response = str::from_utf8(&buf)
            .map_err(|e| LibiguanaError::Utf8Error(e))?
            .to_string();

        Ok(response)
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        // We should probably check for errors here?
        let _ = self.jimulator_process.kill();
    }
}
