use crate::{
    pin::Pin,
    tab::{Tab, TabKind},
};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub enum Command {
    // Other
    Drop,
    Error(Box<dyn Debug + Send>),
    Sync(Vec<Pin>, Vec<Tab>),

    // Pin
    CreatePin(PathBuf),
    DeletePin(usize),
    ChangePinName(usize, String),
    ChangePinDirectory(usize, PathBuf),
    ChangePinPosition(usize, usize, Pin),
    NewPin(Option<usize>, Pin),

    // Tab
    CreateTab(TabKind, PathBuf),
    DeleteTab(Tab),
    ChangeTabDirectory(usize, Tab, Option<PathBuf>),
    ChangeTabName(usize, Tab, String),
    ChangeTabPosition(usize, usize, Tab),
    NewTab(Option<usize>, Tab),
}
