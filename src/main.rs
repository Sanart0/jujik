use jujik::{
    commands::Command, controller::JujikController, error::CustomError, model::JujikModel,
    view::JujikView,
};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{fs::File, sync::mpsc};

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

    let (controller_tx, controller_rx) = mpsc::channel::<Command>();
    let (model_tx, model_rx) = mpsc::channel::<Command>();
    let (view_tx, view_rx) = mpsc::channel::<Command>();

    let controller = JujikController::new(model_tx, view_tx, controller_rx);
    let model = JujikModel::new(controller_tx.clone(), model_rx);
    let view = JujikView::new(controller_tx, view_rx);

    let controller_handler = controller.run();
    let model_handler = model.run();
    let view_handler = view.run();

    let jujik_handlers = vec![view_handler, model_handler, controller_handler];

    for handler in jujik_handlers {
        match handler.join() {
            Ok(thread_return) => {
                if let Err(err) = thread_return {
                    Err(err)?;
                }
            }
            Err(err) => Err(CustomError::Thread(err))?,
        }
    }

    Ok(())
}
