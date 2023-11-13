use libiguana::Environment;
use unicorn_engine::unicorn_const::{Arch, Mode};

fn main() {
    let kmd = include_str!("hello.kmd");

    let mut env =
        Environment::new(Arch::ARM, Mode::LITTLE_ENDIAN).expect("Unable to setup environment!");

    env.load_kmd(kmd);
}
