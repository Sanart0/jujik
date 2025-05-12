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
    DeletePin(Pin),
    NewPin(Pin),

    // Tab
    CreateTab(TabKind, PathBuf),
    DeleteTab(Tab),
    ChangeTabDirectory(usize, Tab, PathBuf),
    ChangeTabDirectoryBack(usize, Tab),
    NewTab(Option<usize>, Tab),
}
