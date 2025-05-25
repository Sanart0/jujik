use jujik::{
    commands::Command, controller::JujikController, error::JujikError, model::JujikModel,
    view::JujikView,
};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{
    fs::{self, File},
    process,
    sync::mpsc,
};

fn main() -> Result<(), JujikError> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
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

    test();

    let (controller_tx, controller_rx) = mpsc::channel::<Command>();
    let (model_tx, model_rx) = mpsc::channel::<Command>();
    let (view_tx, view_rx) = mpsc::channel::<Command>();

    let controller = JujikController::new(model_tx, view_tx, controller_rx);
    let model = JujikModel::new(controller_tx.clone(), model_rx);
    let view = JujikView::new(controller_tx, view_rx);

    let controller_handler = controller.run();
    let model_handler = model.run();
    let view_handler = view.run();

    controller_handler?.join()??;
    model_handler?.join()??;
    view_handler?.join()??;

    Ok(())
}

fn test() {
    fs::remove_dir_all("/home/sanart0/KPI/4/IPZ-Kursach/jujik/test/").unwrap();

    process::Command::new("mkdir")
        .arg("test/")
        .output()
        .unwrap();

    process::Command::new("touch")
        .arg("test/test_1.txt")
        .output()
        .unwrap();
    process::Command::new("touch")
        .arg("test/test_2")
        .output()
        .unwrap();
    process::Command::new("touch")
        .arg("test/test_3")
        .output()
        .unwrap();

    process::Command::new("mkdir")
        .arg("test/test_dir_1")
        .output()
        .unwrap();

    process::Command::new("touch")
        .arg("test/test_dir_1/test_4")
        .output()
        .unwrap();
    process::Command::new("touch")
        .arg("test/test_dir_1/test_5")
        .output()
        .unwrap();

    fs::write("test/test_1.txt", "Some Text Content").unwrap();
    fs::write("test/test_2", "Some Text Content\nWith New Line").unwrap();
}
