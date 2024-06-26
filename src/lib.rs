use std::{
    array::TryFromSliceError,
    collections::HashMap,
    path::Path,
    process::{Child, Command, Stdio},
    str,
    sync::{Arc, Mutex},
};

mod aasm_output;
pub mod arm_decoder;
mod error;
mod kmdparse_types;
mod reader_writer;
mod registers;
mod status;
mod uniffi_array;

use kmdparse::{parse_kmd, token::Token, word::Word};
use kmdparse_types::token::KmdparseToken;
use reader_writer::ReaderWriter;

use crate::status::BoardState;

pub use self::aasm_output::AasmOutput;
pub use self::error::LibiguanaError;
pub use self::registers::Registers;
pub use self::status::Status;

uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct IguanaEnvironment {
    /// The jimulator process that `IguanaEnvironment` controls. This process is killed on `Drop`.
    jimulator_process: Arc<Mutex<Child>>,

    /// The currently loaded `.kmd` file
    current_kmd: Arc<Mutex<Option<Vec<KmdparseToken>>>>,

    /// The path to an `aasm` binary
    aasm_path: String,

    /// The path to the `mnemonics` file required by `aasm`.
    mnemonics_path: String,

    /// Currently defined traps, in the format [memory address : trap number]
    traps: Arc<Mutex<HashMap<u32, u8>>>,

    /// The used trap numbers, with `true` meaning used and `false` meaning unused.
    used_trap_numbers: Arc<Mutex<[bool; u8::MAX as usize]>>,
}

#[uniffi::export]
impl IguanaEnvironment {
    /// Creates a new environment.
    ///
    /// While `jimulator_path` can be anything that resolves to a jimulator executable (by that, I
    /// mean you can just put `jimulator` if it is in your PATH), `aasm_path` must be an absolute
    /// path to an `aasm` executable. There must also be a file called `mnemonics` in the same
    /// directory.
    #[uniffi::constructor]
    pub fn new(
        jimulator_path: &str,
        aasm_path: String,
        mnemonics_path: String,
    ) -> Result<Self, LibiguanaError> {
        if !Path::new(&aasm_path).exists() {
            return Err(LibiguanaError::AasmDoesNotExist);
        }

        if !Path::new(&mnemonics_path).exists() {
            return Err(LibiguanaError::MnemonicsDoesNotExist);
        }

        let jimulator_process = Command::new(jimulator_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let jimulator_arc_mutex = Arc::new(Mutex::new(jimulator_process));

        let traps = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            jimulator_process: jimulator_arc_mutex,
            current_kmd: Arc::new(Mutex::new(None)),
            aasm_path,
            mnemonics_path,
            traps,
            used_trap_numbers: Arc::new(Mutex::new([false; u8::MAX as usize])),
        })
    }

    pub fn compile_aasm(&self, aasm_path: &str) -> Result<AasmOutput, LibiguanaError> {
        let aasm_command = Command::new(&self.aasm_path)
            .args(["-lk", "/dev/stderr", "-m", &self.mnemonics_path, aasm_path])
            // .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Write the aasm string into aasm
        // aasm_command
        //     .stdin
        //     .as_mut()
        //     .ok_or(LibiguanaError::NoStdin)?
        //     .write_all(aasm_string.as_bytes())?;

        let output = aasm_command.wait_with_output()?;

        let kmd = String::from_utf8(output.stderr)?;
        let aasm_terminal = String::from_utf8(output.stdout)?;

        let aasm_output = AasmOutput { kmd, aasm_terminal };

        Ok(aasm_output)
    }

    pub fn continue_execution(&self) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        ReaderWriter::write(&[0b0010_0010], &mut process)?;

        Ok(())
    }

    pub fn create_breakpoint(&self, memory_address: u32) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();
        let mut traps = self.traps.lock().unwrap();
        let mut used_trap_numbers = self.used_trap_numbers.lock().unwrap();

        let trap_number: u8 = used_trap_numbers
            .iter()
            .position(|is_used| !is_used)
            .ok_or(LibiguanaError::TooManyTraps)? as u8;

        // Initial define trap command
        ReaderWriter::write(&[0b0011_0000], &mut process)?;

        ReaderWriter::write(
            &[
                trap_number,
                0b1111_1111, // Trap conditions
                0b0000_1111, // Transfer size mask (all)
            ],
            &mut process,
        )?;

        // Trap address A and B
        ReaderWriter::write(&memory_address.to_le_bytes(), &mut process)?;
        ReaderWriter::write(&u32::MAX.to_le_bytes(), &mut process)?;

        // Data address A and B (unused by Iguana)
        ReaderWriter::write(&[0; 16], &mut process)?;

        traps.insert(memory_address, trap_number);
        used_trap_numbers[trap_number as usize] = true;

        Ok(())
    }

    pub fn current_kmd(&self) -> Option<Vec<KmdparseToken>> {
        self.current_kmd.lock().unwrap().clone()
    }

    /// Kills the underlying jimulator process. This function should not be used from within Rust -
    /// `IguanaEnvironment` implements `Drop` and handles killing the process for you. This exists
    /// because for some reason `Drop` isn't working through `uniffi`.
    pub fn kill_jimulator(&self) -> Result<(), LibiguanaError> {
        self.jimulator_process.lock().unwrap().kill()?;

        Ok(())
    }

    /// Loads the given .kmd file. [`kmd`] is an unparsed string - parsing is handled by this
    /// function.
    pub fn load_kmd(&self, kmd: &str) -> Result<(), LibiguanaError> {
        let mut current_kmd = self.current_kmd.lock().unwrap();

        let parsed = parse_kmd(kmd).map_err(|_| LibiguanaError::ParseError)?.1;

        for token in &parsed {
            if let Token::Line(line) = token {
                if let (Some(word_wrapper), Some(memory_address)) =
                    (line.word.clone(), line.memory_address)
                {
                    match word_wrapper {
                        Word::Instruction(instruction) => {
                            self.write_memory(&instruction, memory_address)?
                        }
                        Word::Data(data) => self.write_memory(&data, memory_address)?,
                    };
                }
            }
        }

        let converted_kmd = parsed
            .into_iter()
            .map(|token| KmdparseToken::from(token))
            .collect::<Vec<_>>();

        *current_kmd = Some(converted_kmd);

        Ok(())
    }

    // Pauses execution.
    pub fn pause(&self) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        ReaderWriter::write(&[0b0010_0010], &mut process)?;

        Ok(())
    }

    pub fn ping(&self) -> Result<String, LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        ReaderWriter::write(&[0b0000_0001], &mut process)?;

        let mut buf = [0; 4];

        ReaderWriter::read_exact(&mut buf, &mut process)?;

        let response = str::from_utf8(&buf)?.to_string();

        Ok(response)
    }

    pub fn read_memory(&self, address: u32) -> Result<u32, LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        // Write memory transfer command (mem space, read, 32 bit)
        ReaderWriter::write(&[0b01_00_1_010], &mut process)?;

        // Write address
        ReaderWriter::write(&address.to_le_bytes(), &mut process)?;

        // Write length (1)
        ReaderWriter::write(&1_u16.to_le_bytes(), &mut process)?;

        let mut buf = [0; 4];
        ReaderWriter::read_exact(&mut buf, &mut process)?;

        Ok(u32::from_le_bytes(buf))
    }

    pub fn registers(&self) -> Result<Registers, LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        // Write memory transfer command (reg space, read, 32 bit)
        ReaderWriter::write(&[0b01_01_1_010], &mut process)?;

        // Write address (0, it's what KoMo2 does)
        ReaderWriter::write(&0_u32.to_le_bytes(), &mut process)?;

        // Write length (16)
        ReaderWriter::write(&16_u16.to_le_bytes(), &mut process)?;

        let mut buf = [0; 64];

        ReaderWriter::read_exact(&mut buf, &mut process)?;

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

    pub fn remove_breakpoint(&self, memory_address: u32) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();
        let mut traps = self.traps.lock().unwrap();
        let mut used_trap_numbers = self.used_trap_numbers.lock().unwrap();

        let trap_number = traps
            .remove(&memory_address)
            .ok_or(LibiguanaError::NoTrapForAddress(memory_address))?;

        // Send word A (all 0s - disable)
        ReaderWriter::write(&[0; 4], &mut process)?;
        ReaderWriter::write(&(trap_number as u32).to_le_bytes(), &mut process)?;

        used_trap_numbers[trap_number as usize] = false;

        Ok(())
    }

    pub fn reset(&self) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();
        let mut traps = self.traps.lock().unwrap();
        let mut used_trap_numbers = self.used_trap_numbers.lock().unwrap();

        ReaderWriter::write(&[0b0000_0100], &mut process)?;

        traps.clear();
        *used_trap_numbers = [false; u8::MAX as usize];

        Ok(())
    }

    /// Starts execution, with the given step limit. If the step limit is 0, the emulator will
    /// execute indefinitely.
    pub fn start_execution(&self, steps: u32) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        ReaderWriter::write(&[0b1011_0000], &mut process)?;
        ReaderWriter::write(&steps.to_le_bytes(), &mut process)?;

        Ok(())
    }

    pub fn stop_execution(&self) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        ReaderWriter::write(&[0b0010_0001], &mut process)?;

        Ok(())
    }

    pub fn terminal_messages(&self) -> Result<Vec<u8>, LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        let mut length = 1;
        let mut output = Vec::new();

        while length != 0 {
            ReaderWriter::write(&[0b0001_0011, 0, 32], &mut process)?;

            let mut len_buf = [0; 1];

            ReaderWriter::read_exact(&mut len_buf, &mut process)?;

            length = len_buf[0];

            if length == 0 {
                break;
            }

            let mut buf = vec![0; length as usize];

            ReaderWriter::read_exact(&mut buf, &mut process)?;

            output.append(&mut buf);
        }

        Ok(output)
    }

    pub fn status(&self) -> Result<BoardState, LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        ReaderWriter::write(&[0b0010_0000], &mut process)?;

        let mut buf = [0; 9];

        ReaderWriter::read_exact(&mut buf, &mut process)?;

        let steps_remaining = u32::from_le_bytes(buf[1..5].try_into()?);
        let steps_since_reset = u32::from_le_bytes(buf[5..9].try_into()?);

        let status = BoardState {
            status: Status::try_from(buf[0]).map_err(|_| LibiguanaError::InvalidStatus(buf[0]))?,
            steps_remaining,
            steps_since_reset,
        };

        Ok(status)
    }

    pub fn traps(&self) -> HashMap<u32, u8> {
        self.traps.lock().unwrap().clone()
    }

    pub fn write_to_terminal(&self, message: &[u8]) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        // jimulator only takes one byte as length, so we have to chunk the input into chunks of 256
        let chunks = message.chunks(u8::MAX as usize);

        for chunk in chunks {
            ReaderWriter::write(&[0b0001_0010], &mut process)?;

            // Terminal 0
            ReaderWriter::write(&[0], &mut process)?;

            // Casting here should be fine, a chunk can't be bigger than u8::MAX
            ReaderWriter::write(&[chunk.len() as u8], &mut process)?;

            ReaderWriter::write(chunk, &mut process)?;

            // jimulator returns 0 after every write for some reason
            ReaderWriter::read_exact(&mut [0], &mut process)?;
        }

        Ok(())
    }

    fn write_memory(&self, word: &[u8], address: u32) -> Result<(), LibiguanaError> {
        let mut process = self.jimulator_process.lock().unwrap();

        if word.is_empty() {
            return Ok(());
        }

        // Write memory transfer command (mem space, write, 8 bit)
        ReaderWriter::write(&[0b01_00_0_000], &mut process)?;

        // Write address
        ReaderWriter::write(&address.to_le_bytes(), &mut process)?;

        let num_elements: u16 = word.len().try_into()?;

        // Write number of elements (number of elements in address slice)
        ReaderWriter::write(&num_elements.to_le_bytes(), &mut process)?;

        ReaderWriter::write(word, &mut process)?;

        Ok(())
    }
}

impl Drop for IguanaEnvironment {
    fn drop(&mut self) {
        println!("Drop called!");
        let mut process = self.jimulator_process.lock().unwrap();

        if let Err(e) = process.kill() {
            eprintln!("Failed to kill jimulator process: {e:?}");
        }

        if let Err(e) = process.wait() {
            eprintln!("Error occured while waiting for jimulator process to quit: {e:?}");
        }
    }
}
