use std::{thread::sleep, time::Duration};

use libiguana::{Environment, Status};

fn main() {
    let kmd = include_str!("02-meadow.kmd");

    let mut env = Environment::new().expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");

    for i in 0..0x100 {
        if i % 4 != 0 {
            continue;
        }

        let mem = env.read_memory(i).expect("Failed to read memory!");
        let mem_u32 = u32::from_le_bytes(mem);
        println!("{i:#08x}: {mem_u32:#08x}");
    }

    let registers = env.registers().expect("Failed to get registers!");
    println!("{registers:?}");

    let status = env.status().expect("Failed to get status!");
    println!("{status:?}");

    env.start(10000).expect("Failed to start!");

    loop {
        let status = env.status().expect("Failed to get status!");

        if status.status == Status::Stopped {
            break;
        }

        let registers = env.registers().expect("Failed to get registers!");

        println!("{status:?}");
        println!("{registers:?}");
    }

    let status = env.status().expect("Failed to get status!");
    println!("{status:?}");

    let registers = env.registers().expect("Failed to get registers!");
    println!("{registers:?}");

    let terminal = env.terminal_messages().expect("Failed to read!");

    println!("{terminal}");
}
