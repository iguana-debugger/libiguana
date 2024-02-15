use std::{thread::sleep, time::Duration};

use libiguana::Environment;

fn main() {
    let kmd = include_str!("hello.kmd");

    let mut env = Environment::new().expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");

    let registers = env.registers().expect("Failed to get registers!");
    println!("{registers:?}");

    let status = env.status().expect("Failed to get status!");
    println!("{status:?}");

    env.start().expect("Failed to start!");

    sleep(Duration::from_secs(1));

    let status = env.status().expect("Failed to get status!");
    println!("{status:?}");

    let registers = env.registers().expect("Failed to get registers!");
    println!("{registers:?}");

    let terminal = env.terminal_messages().expect("Failed to read!");

    println!("{terminal}");
}
