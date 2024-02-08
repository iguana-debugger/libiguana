use std::{
    io::{self, Read, Write},
    process::{Child, Command, Stdio},
    str,
};

mod error;
use kmdparse::{parse_kmd, token::Token};

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

    /// Loads the given .kmd file. [`kmd`] is an unparsed string - parsing is handled by this
    /// function.
    pub fn load_kmd(&mut self, kmd: &str) -> Result<(), LibiguanaError> {
        let parsed = parse_kmd(kmd).map_err(|_| LibiguanaError::ParseError)?.1;

        for token in parsed {
            if let Token::Line(line) = token {
                if let Some(word) = line.word {
                    self.write_memory(word, line.memory_address)?;
                }
            }
        }

        Ok(())
    }

    fn write_memory(&mut self, word: u32, address: u32) -> Result<(), LibiguanaError> {
        let mut stdin = self
            .jimulator_process
            .stdin
            .as_ref()
            .ok_or(LibiguanaError::NoStdin)?;

        // Write memory transfer command (mem space, write, 32 bit)
        stdin
            .write(&[0b01_00_0_010])
            .map_err(|e| LibiguanaError::IO(e))?;

        // Write address
        stdin
            .write(&address.to_le_bytes())
            .map_err(|e| LibiguanaError::IO(e))?;

        // Write number of elements (always 1 in this case)
        stdin
            .write(&1_u16.to_le_bytes())
            .map_err(|e| LibiguanaError::IO(e))?;

        stdin
            .write(&word.to_le_bytes())
            .map_err(|e| LibiguanaError::IO(e))?;

        Ok(())
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        // We should probably check for errors here?
        let _ = self.jimulator_process.kill();
    }
}
