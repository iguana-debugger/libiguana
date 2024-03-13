use libiguana::IguanaEnvironment;

fn main() {
    let assembly = include_str!("hello.s");

    let env = IguanaEnvironment::new("jimulator", "/usr/local/bin/aasm".to_string())
        .expect("Unable to setup environment!");

    let kmd = env
        .compile_aasm(assembly)
        .expect("Failed to compile assembly!");

    println!("{}", kmd.aasm_terminal);
    println!("{}", kmd.kmd);
}
