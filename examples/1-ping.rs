use libiguana::Environment;

fn main() {
    let mut env = Environment::new().expect("Unable to setup environment!");

    let ping_res = env.ping().expect("Ping failed!");

    println!("{ping_res}");
}
