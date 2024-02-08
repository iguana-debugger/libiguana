use libiguana::Environment;

fn main() {
    let kmd = include_str!("hello.kmd");

    let env = Environment::new().expect("Unable to setup environment!");
}
