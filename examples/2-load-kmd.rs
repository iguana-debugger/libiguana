use libiguana::IguanaEnvironment;

fn main() {
    let kmd = include_str!("hello.kmd");

    let env = IguanaEnvironment::new().expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");
}
