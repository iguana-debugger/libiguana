use std::process::exit;

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
            .mem_map(0x0, 2 * 1024 * 1024, Permission::ALL)
            .unwrap();

        let mut parsed_bytes: Vec<u8> = vec![];

        for token in parsed_kmd {
            if let Token::Line(line) = token {
                if let Some(word) = line.word {
                    let mut bytes = word.to_le_bytes();
                    bytes.reverse();

                    for byte in bytes {
                        parsed_bytes.push(byte);
                    }

                    println!("Writing {:X} at {:X}", word, line.memory_address);

                    self.unicorn
                        .mem_write(line.memory_address as u64, &bytes)
                        .unwrap();
                }
            }
        }

        let uc_mem = self.unicorn.mem_read_as_vec(0, 2 * 1024 * 1024).unwrap();

        for i in 0..0x34 {
            println!("0x{:X}: {} {}", i, parsed_bytes[i], uc_mem[i]);
        }

        self.unicorn
            .add_intr_hook(|uc, num| {
                println!("Interrupt {num} called!");
                if num == 2 {
                    println!("{}", uc.reg_read(RegisterARM::R0).unwrap());
                    exit(0);
                }
            })
            .unwrap();

        self.unicorn.emu_start(0, 2 * 1024 * 1024, 0, 0).unwrap();
    }
}
