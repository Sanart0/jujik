use eframe::{run_native, NativeOptions};
use jujik::jujik::Jujik;

fn main() {
    let jujik = Jujik::new();

    let native_options = NativeOptions::default();
    let _ = run_native("Jujik", native_options, Box::new(|cc| Ok(Box::new(jujik))));
}
