use jujik::{controller::Jujik, error::CustomError, model::JujikModel, view::JujikView};
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

    let controller = Jujik::new();
    let model = JujikModel::new();
    let view = JujikView::new();

    let controller_handler = controller.run();
    let model_handler = model.run();
    let view_handler = view.run();

    view_handler.join()??;
    model_handler.join()??;
    controller_handler.join()??;

    Ok(())
}
