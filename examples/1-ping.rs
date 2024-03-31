use libiguana::IguanaEnvironment;

fn main() {
    let env = IguanaEnvironment::new(
        "jimulator",
        "/usr/local/bin/aasm".to_string(),
        "/usr/local/bin/mnemonics".to_string(),
    )
    .expect("Unable to setup environment!");

    let ping_res = env.ping().expect("Ping failed!");

    println!("{ping_res}");
}
