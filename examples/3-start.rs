use libiguana::{IguanaEnvironment, Status};

fn main() {
    let kmd = include_str!("hello.kmd");

    let env = IguanaEnvironment::new(
        "jimulator",
        "/usr/local/bin/aasm".to_string(),
        "/usr/local/bin/mnemonics".to_string(),
    )
    .expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");

    env.start_execution(0).expect("Failed to start!");

    loop {
        let status = env.status().expect("Failed to get status!");

        if status.status == Status::Stopped || status.status == Status::Finished {
            break;
        }

        let terminal = env.terminal_messages().expect("Failed to read!");
        let terminal_string = String::from_utf8(terminal).expect("Failed to convert from UTF8!");

        print!("{terminal_string}");
    }

    let status = env.status().expect("Failed to get status!");
    println!("{status:?}");

    let registers = env.registers().expect("Failed to get registers!");
    println!("{registers:?}");
}
