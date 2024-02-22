use libiguana::IguanaEnvironment;

fn main() {
    let env = IguanaEnvironment::new("jimulator").expect("Unable to setup environment!");

    let ping_res = env.ping().expect("Ping failed!");

    println!("{ping_res}");
}
