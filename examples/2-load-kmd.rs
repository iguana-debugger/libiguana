use libiguana::IguanaEnvironment;

fn main() {
    let kmd = include_str!("hello.kmd");

    let env = IguanaEnvironment::new(
        "jimulator",
        "/usr/local/bin/aasm".to_string(),
        "/usr/local/bin/mnemonics".to_string(),
    )
    .expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");
}
