use std::env;

use log::LevelFilter;
use rust_nes_emulator::{
    nes::{ActionNES, NES},
    screen::run,
};

fn main() {
    setup_logging().expect("Logger setup failed");
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("Pass .nes file path to run");
    // let mut nes = TraceNes::new(0);
    let mut nes = ActionNES::new();
    nes.load_from_path(path).expect("Failed to load path");
    nes.reset();
    run(nes)
}

fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        // Format each log message
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        // Filter logs based on level
        .level(LevelFilter::Debug)
        // Output to stdout
        // .chain(std::io::stdout())
        // Output to a file
        .chain(fern::log_file("output.log")?)
        // Apply settings
        .apply()?;
    Ok(())
}
