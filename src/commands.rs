use crate::pin::Pin;
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub enum Command {
    Drop,

    // Pin
    NewPin(String),
    CreatePin(PathBuf),
    ShowPin(Pin),
    ErrorPin(Box<dyn Debug + Send>),
    // Tab

    // External Storage
}
