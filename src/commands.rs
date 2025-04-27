use crate::{entity::Entity, pin::Pin, tab::Tab};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub enum Command {
    Drop,
    Error(Box<dyn Debug + Send>),

    // Pin
    NewPin(PathBuf),
    DeletePin(Pin),
    ShowPin(Pin),

    // Tab
    NewTab(PathBuf),
    DeleteTab(Tab),
    CreateTab(PathBuf),
    ChangeDirectory(Tab, PathBuf),
    ShowTab(Tab),
    // External Storage
}
