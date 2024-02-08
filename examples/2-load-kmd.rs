use libiguana::Environment;

fn main() {
    let kmd = include_str!("hello.kmd");

    let mut env = Environment::new().expect("Unable to setup environment!");

    env.load_kmd(kmd).expect("Load kmd failed!");
}
