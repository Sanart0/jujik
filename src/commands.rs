use crate::{pin::Pin, tab::Tab};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub enum Command {
    Drop,
    Error(Box<dyn Debug + Send>),
    Sync(Vec<Pin>, Vec<Tab>),

    // Pin
    NewPin(PathBuf),
    DeletePin(Pin),
    ShowPin(Pin),

    // Tab
    NewTab(PathBuf),
    DeleteTab(Tab),
    ChangeDirectory(Tab, PathBuf),
    ShowTab(Tab),
    // External Storage
}
