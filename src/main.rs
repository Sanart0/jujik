use jujik::{
    commands::Command, controller::JujikController, error::JujikError, model::JujikModel,
    view::JujikView,
};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{fs::File, sync::mpsc};

fn main() -> Result<(), JujikError> {
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
