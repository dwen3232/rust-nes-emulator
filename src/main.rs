use std::env;

use rust_nes_emulator::screen::run;
use rust_nes_emulator::screen::emscripten::run_emscripten;

fn main() {
    run_emscripten("game_roms/pacman.nes")
    // let args: Vec<String> = env::args().collect();
    // if let Some(path) = args.get(1) {
    //     // run(path);
    //     run_emscripten(path)
    // } else {
    //     println!("Pass .nes file path to run")
    // }
}
