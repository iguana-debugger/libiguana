use std::{
    array::TryFromSliceError,
    io::{self, Read, Write},
    process::{Child, Command, Stdio},
    str,
};

mod error;
mod registers;
mod status;
use kmdparse::{parse_kmd, token::Token};

use crate::status::BoardState;

pub use self::error::LibiguanaError;
pub use self::registers::Registers;
pub use self::status::Status;

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

    /// Loads the given .kmd file. [`kmd`] is an unparsed string - parsing is handled by this
    /// function.
    pub fn load_kmd(&mut self, kmd: &str) -> Result<(), LibiguanaError> {
        let parsed = parse_kmd(kmd).map_err(|_| LibiguanaError::ParseError)?.1;

        for token in parsed {
            if let Token::Line(line) = token {
                if let Some(word) = line.word {
                    self.write_memory(&word, line.memory_address)?;
                }
            }
        }

        Ok(())
    }

    pub fn ping(&mut self) -> Result<String, LibiguanaError> {
        self.write(&[0b0000_0001])?;

        let mut buf = [0; 4];

        self.read_exact(&mut buf)?;

        let response = str::from_utf8(&buf)?.to_string();

        Ok(response)
    }

    pub fn read_memory(&mut self, address: u32) -> Result<[u8; 4], LibiguanaError> {
        // Write memory transfer command (mem space, read, 32 bit)
        self.write(&[0b01_00_1_010])?;

        // Write address
        self.write(&address.to_le_bytes())?;

        // Write length (1)
        self.write(&1_u16.to_le_bytes())?;

        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn registers(&mut self) -> Result<Registers, LibiguanaError> {
        // Write memory transfer command (reg space, read, 32 bit)
        self.write(&[0b01_01_1_010])?;

        // Write address (0, it's what KoMo2 does)
        self.write(&0_u32.to_le_bytes())?;

        // Write length (16)
        self.write(&16_u16.to_le_bytes())?;

        let mut buf = [0; 64];

        self.read_exact(&mut buf)?;

        // Convert the buf of u8s into a buf of u32s. This code is a bit clunky because array_chunks
        // isn't in stable yet, so we have to manually try_into all of the slices. This also
        // involves collecting the try_into results into a Vec so that we can handle errors.
        let u32_buf = buf
            .chunks_exact(4)
            .map(|chunk| TryInto::<[u8; 4]>::try_into(chunk))
            .collect::<Result<Vec<[u8; 4]>, TryFromSliceError>>()?
            .iter()
            .map(|array_chunk| u32::from_le_bytes(*array_chunk))
            .collect::<Vec<_>>();

        // Just in case, we check the length here. Better safe than sorry :)
        if u32_buf.len() != 16 {
            return Err(LibiguanaError::InvalidRegisterBufferLength(u32_buf.len()));
        }

        let registers = Registers {
            r0: u32_buf[0],
            r1: u32_buf[1],
            r2: u32_buf[2],
            r3: u32_buf[3],
            r4: u32_buf[4],
            r5: u32_buf[5],
            r6: u32_buf[6],
            r7: u32_buf[7],
            r8: u32_buf[8],
            r9: u32_buf[9],
            r10: u32_buf[10],
            r11: u32_buf[11],
            r12: u32_buf[12],
            r13: u32_buf[13],
            r14: u32_buf[14],
            pc: u32_buf[15],
        };

        Ok(registers)
    }

    pub fn start(&mut self, steps: u32) -> Result<(), LibiguanaError> {
        self.write(&[0b1011_0000])?;
        self.write(&steps.to_le_bytes())?;

        Ok(())
    }

    pub fn terminal_messages(&mut self) -> Result<String, LibiguanaError> {
        let mut length = 1;
        let mut output = String::new();

        while length != 0 {
            self.write(&[0b0001_0011, 0, 32])?;

            println!("Write request sent");

            let mut len_buf = [0; 1];

            self.read_exact(&mut len_buf)?;

            println!("Length read");

            length = len_buf[0];

            println!("{length}");

            if length == 0 {
                break;
            }

            let mut buf = vec![0; length as usize];

            self.read_exact(&mut buf)?;

            println!("String fragment read");

            let read_str = str::from_utf8(&buf)?;

            output.push_str(read_str);
        }

        Ok(output)
    }

    pub fn status(&mut self) -> Result<BoardState, LibiguanaError> {
        self.write(&[0b0010_0000])?;

        let mut buf = [0; 9];

        self.read_exact(&mut buf)?;

        let steps_remaining = u32::from_le_bytes(buf[1..5].try_into()?);
        let steps_since_reset = u32::from_le_bytes(buf[5..9].try_into()?);

        let status = BoardState {
            status: Status::try_from(buf[0]).map_err(|_| LibiguanaError::InvalidStatus(buf[0]))?,
            steps_remaining: steps_remaining,
            steps_since_reset: steps_since_reset,
        };

        Ok(status)
    }

    /// Reads from the jimulator process using read_exact.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), LibiguanaError> {
        self.jimulator_process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_exact(buf)?;

        Ok(())
    }

    /// Reads from the jimulator process using read_until_end.
    fn read_to_end(&mut self) -> Result<Vec<u8>, LibiguanaError> {
        let mut buf = Vec::new();

        self.jimulator_process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_to_end(&mut buf)?;

        Ok(buf)
    }

    /// Writes the given byte array to the jimulator process.
    fn write(&mut self, payload: &[u8]) -> Result<(), LibiguanaError> {
        self.jimulator_process
            .stdin
            .as_ref()
            .ok_or(LibiguanaError::NoStdin)?
            .write_all(payload)?;

        Ok(())
    }

    fn write_memory(&mut self, word: &[u8], address: u32) -> Result<(), LibiguanaError> {
        if word.is_empty() {
            return Ok(());
        }

        let word_rev = word.iter().map(|byte| *byte).rev().collect::<Vec<_>>();

        println!("Writing {word_rev:?} to {address:#08x}");

        // Write memory transfer command (mem space, write, 8 bit)
        self.write(&[0b01_00_0_000])?;

        // Write address
        self.write(&address.to_le_bytes())?;

        let num_elements: u16 = word_rev.len().try_into()?;

        // Write number of elements (number of elements in address slice)
        self.write(&num_elements.to_le_bytes())?;

        self.write(&word_rev)?;

        Ok(())
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        // We should probably check for errors here?
        let _ = self.jimulator_process.kill();
    }
}
