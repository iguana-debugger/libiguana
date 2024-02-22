use std::{
    array::TryFromSliceError,
    io::{Read, Write},
    process::{Child, Command, Stdio},
    str,
    sync::{Arc, Mutex},
};

mod error;
mod registers;
mod status;
mod uniffi_array;
use kmdparse::{parse_kmd, token::Token, word::Word};

use crate::status::BoardState;

pub use self::error::LibiguanaError;
pub use self::registers::Registers;
pub use self::status::Status;

uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct IguanaEnvironment {
    jimulator_process: Arc<Mutex<Child>>,
}

#[uniffi::export]
impl IguanaEnvironment {
    #[uniffi::constructor]
    pub fn new(path: &str) -> Result<Self, LibiguanaError> {
        let jimulator_process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let arc_mutex = Arc::new(Mutex::new(jimulator_process));

        Ok(Self {
            jimulator_process: arc_mutex,
        })
    }

    /// Loads the given .kmd file. [`kmd`] is an unparsed string - parsing is handled by this
    /// function.
    pub fn load_kmd(&self, kmd: &str) -> Result<(), LibiguanaError> {
        let parsed = parse_kmd(kmd).map_err(|_| LibiguanaError::ParseError)?.1;

        for token in parsed {
            if let Token::Line(line) = token {
                if let (Some(word_wrapper), Some(memory_address)) = (line.word, line.memory_address)
                {
                    match word_wrapper {
                        Word::Instruction(word) => self.write_memory(&word, memory_address)?,
                        Word::Data(data) => self.write_memory(&data, memory_address)?,
                    };
                }
            }
        }

        Ok(())
    }

    pub fn ping(&self) -> Result<String, LibiguanaError> {
        self.write(&[0b0000_0001])?;

        let mut buf = [0; 4];

        self.read_exact(&mut buf)?;

        let response = str::from_utf8(&buf)?.to_string();

        Ok(response)
    }

    pub fn read_memory(&self, address: u32) -> Result<[u8; 4], LibiguanaError> {
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

    pub fn registers(&self) -> Result<Registers, LibiguanaError> {
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

    pub fn start(&self, steps: u32) -> Result<(), LibiguanaError> {
        self.write(&[0b1011_0000])?;
        self.write(&steps.to_le_bytes())?;

        Ok(())
    }

    pub fn terminal_messages(&self) -> Result<String, LibiguanaError> {
        let mut length = 1;
        let mut output = String::new();

        while length != 0 {
            self.write(&[0b0001_0011, 0, 32])?;

            let mut len_buf = [0; 1];

            self.read_exact(&mut len_buf)?;

            length = len_buf[0];

            if length == 0 {
                break;
            }

            let mut buf = vec![0; length as usize];

            self.read_exact(&mut buf)?;

            let read_str = str::from_utf8(&buf)?;

            output.push_str(read_str);
        }

        Ok(output)
    }

    pub fn status(&self) -> Result<BoardState, LibiguanaError> {
        self.write(&[0b0010_0000])?;

        let mut buf = [0; 9];

        self.read_exact(&mut buf)?;

        let steps_remaining = u32::from_le_bytes(buf[1..5].try_into()?);
        let steps_since_reset = u32::from_le_bytes(buf[5..9].try_into()?);

        let status = BoardState {
            status: Status::try_from(buf[0]).map_err(|_| LibiguanaError::InvalidStatus(buf[0]))?,
            steps_remaining,
            steps_since_reset,
        };

        Ok(status)
    }

    pub fn write_to_terminal(&self, message: &str) -> Result<(), LibiguanaError> {
        // Komodo almost definitely expects ASCII, it'd be interesting to see what happens when we
        // send something that doesn't directly translate from UTF-8 to ASCII (i.e., anything not in
        // ASCII)
        let buf = message.as_bytes();

        // jimulator only takes one byte as length, so we have to chunk the input into chunks of 256
        let chunks = buf.chunks(u8::MAX as usize);

        for chunk in chunks {
            self.write(&[0b0001_0010])?;

            // Terminal 0
            self.write(&[0])?;

            // to_le_bytes doesn't technically guarantee that the length here is one, but since a
            // chunk's length can't be more than one u8 it should never happen.
            self.write(&chunk.len().to_le_bytes())?;

            self.write(chunk)?;

            // jimulator returns 0 after every write for some reason
            self.read_exact(&mut [0])?;
        }

        Ok(())
    }

    /// Reads from the jimulator process using read_until_end.
    fn read_to_end(&self) -> Result<Vec<u8>, LibiguanaError> {
        let mut buf = Vec::new();

        let mut process = self.jimulator_process.lock().unwrap();

        process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_to_end(&mut buf)?;

        Ok(buf)
    }

    /// Writes the given byte array to the jimulator process.
    fn write(&self, payload: &[u8]) -> Result<(), LibiguanaError> {
        let process = self.jimulator_process.lock().unwrap();

        process
            .stdin
            .as_ref()
            .ok_or(LibiguanaError::NoStdin)?
            .write_all(payload)?;

        Ok(())
    }

    fn write_memory(&self, word: &[u8], address: u32) -> Result<(), LibiguanaError> {
        if word.is_empty() {
            return Ok(());
        }

        // Write memory transfer command (mem space, write, 8 bit)
        self.write(&[0b01_00_0_000])?;

        // Write address
        self.write(&address.to_le_bytes())?;

        let num_elements: u16 = word.len().try_into()?;

        // Write number of elements (number of elements in address slice)
        self.write(&num_elements.to_le_bytes())?;

        self.write(word)?;

        Ok(())
    }
}

// uniffi doesn't support &mut [T], so we extract it into a trait here (luckily read_exact is
// internal)
trait ReadExact {
    fn read_exact(&self, buf: &mut [u8]) -> Result<(), LibiguanaError>;
}

impl ReadExact for IguanaEnvironment {
    /// Reads from the jimulator process using read_exact.
    fn read_exact(&self, buf: &mut [u8]) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        process
            .stdout
            .as_mut()
            .ok_or(LibiguanaError::NoStdout)?
            .read_exact(buf)?;

        Ok(())
    }
}

impl Drop for IguanaEnvironment {
    fn drop(&mut self) {
        let mut process = self.jimulator_process.lock().unwrap();

        // We should probably check for errors here?
        let _ = process.kill();
    }
}
