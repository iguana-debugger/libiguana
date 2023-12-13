use std::process::exit;
use std::str;

use kmdparse::parse_kmd;
use kmdparse::token::Token;
use unicorn_engine::{
    unicorn_const::{uc_error, Arch, Mode, Permission},
    RegisterARM, Unicorn,
};

/// An environment for running CPU instructions in. Think of this as your "virtual CPU".
pub struct Environment<'a> {
    unicorn: Unicorn<'a, ()>,
}

impl<'a> Environment<'a> {
    /// Creates a new environment for the given architecture/mode. You probably want [Arch::ARM] and
    /// [Mode::LITTLE_ENDIAN].
    pub fn new(arch: Arch, mode: Mode) -> Result<Environment<'a>, uc_error> {
        let unicorn = Unicorn::new(arch, mode)?;

        Ok(Environment { unicorn })
    }

    pub fn load_kmd(&mut self, kmd: &str) {
        let (_, parsed_kmd) = parse_kmd(kmd).unwrap();

        self.unicorn
            .mem_map(0x0, 1024 * 1024, Permission::ALL)
            .unwrap();

        for token in parsed_kmd {
            if let Token::Line(line) = token {
                if let Some(word) = line.word {
                    self.unicorn
                        .mem_write(line.memory_address as u64, &word.to_le_bytes())
                        .unwrap();
                }
            }
        }

        self.unicorn
            .add_intr_hook(|uc, _| {
                let pc = uc.pc_read().unwrap() - 4;
                let instruction = uc.mem_read_as_vec(pc, 4).unwrap();
                if let Some(interrupt_num) = instruction.first() {
                    println!("SWI {interrupt_num} called!");

                    match interrupt_num {
                        1 => todo!(),
                        2 => exit(0),
                        3 => {
                            let address = uc.reg_read(RegisterARM::R0).expect("Failed to read R0!");

                            let memory = uc
                                .mem_read_as_vec(address, (1024 * 1024 - address) as usize)
                                .expect("Failed to read memory!");

                            let string_bytes = memory
                                .chunks(4)
                                .into_iter()
                                .flat_map(|chunk| chunk.into_iter().rev())
                                .take_while(|byte| **byte != 0)
                                .map(|borrow| *borrow)
                                .collect::<Vec<_>>();

                            for byte in &string_bytes {
                                print!("{:02X} ", byte);
                            }

                            println!();

                            Self::print_str(&string_bytes).expect("Failed to print_str!")
                        }
                        _ => panic!("Invalid SWI: {interrupt_num}"),
                    }
                }
            })
            .unwrap();

        self.unicorn.emu_start(0, 1024 * 1024, 0, 100).unwrap();
    }

    fn print_str(string_bytes: &[u8]) -> Result<(), uc_error> {
        let string = str::from_utf8(&string_bytes).unwrap();

        println!("{string}");

        Ok(())
    }
}
