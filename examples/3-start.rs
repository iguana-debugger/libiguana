use libiguana::{IguanaEnvironment, Status};

fn main() {
    let kmd = include_str!("hello.kmd");

    let env = IguanaEnvironment::new("jimulator").expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");

    env.start_execution(0).expect("Failed to start!");

    loop {
        let status = env.status().expect("Failed to get status!");

        if status.status == Status::Stopped || status.status == Status::Finished {
            break;
        }

        let terminal = env.terminal_messages().expect("Failed to read!");

        print!("{terminal}");
    }

    let status = env.status().expect("Failed to get status!");
    println!("{status:?}");

    let registers = env.registers().expect("Failed to get registers!");
    println!("{registers:?}");
}
