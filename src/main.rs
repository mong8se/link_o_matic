use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(e) = link_o_matic::run(&args) {
        eprintln!("Unexpected: {:?}", e);
        process::exit(2)
    }
}
