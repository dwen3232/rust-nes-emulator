use std::env;

use rust_nes_emulator::screen::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(path) = args.get(1) {
        run(path);
    } else {
        println!("Pass .nes file path to run")
    }
}
