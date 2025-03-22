use eframe::{NativeOptions, run_native};
use jujik::{error::CustomError, controller::Jujik};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;

fn main() -> Result<(), CustomError> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("log.log")?,
        ),
    ])?;

    let jujik = Jujik::new();

    let native_options = NativeOptions::default();
    run_native("Jujik", native_options, Box::new(|_cc| Ok(Box::new(jujik))))?;
    Ok(())
}
