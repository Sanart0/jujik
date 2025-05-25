use crate::{
    config::Config,
    entity::{Entity, find::FindParameters, owner::EntityOwners, permission::EntityPermissions},
    pin::Pin,
    tab::{SortBy, Tab},
};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub enum Command {
    // Other
    Drop,
    Error(Box<dyn Debug + Send>),
    Sync(Vec<Pin>, Vec<Tab>),
    Update,

    // Config
    GetConfig,
    SetConfig(Config),

    // Pin
    CreatePin(PathBuf),
    DeletePin(usize, Pin),
    ChangePinName(usize, Pin, String),
    ChangePinDirectory(usize, Pin, PathBuf),
    ChangePinPosition(usize, usize, Pin),
    NewPin(Option<usize>, Pin),

    // Tab
    CreateEntitys(PathBuf),
    CreateView(PathBuf),
    CreateEditor(PathBuf),
    CreateFinder(FindParameters),
    UpdateTab(usize),
    DeleteTab(usize, Tab),
    ChangeTabName(usize, Tab, String),
    ChangeTabDirectory(usize, Tab, Option<PathBuf>),
    ChangeTabPosition(usize, usize, Tab),
    NewTab(Option<usize>, Tab),
    ChangeEntitysSortBy(usize, Tab, SortBy),

    // Entity
    CreateEntity(usize, Tab, Entity),
    DeleteEntitys(usize, Tab, Vec<Entity>),
    CopyEntitys(usize, Tab, usize, Vec<Entity>, PathBuf),
    MoveEntitys(usize, Tab, usize, Vec<Entity>, PathBuf),
    ChangeEntityName(usize, Tab, usize, Entity, String),
    ChangeEntityExtension(usize, Tab, usize, Entity, String),
    ChangeEntityPermissions(usize, Tab, usize, Entity, EntityPermissions),
    ChangeEntityOwners(usize, Tab, usize, Entity, EntityOwners),
    ChangeEntityContent(usize, Tab, Entity, String),

    // Find
    UpdateFind(usize, Tab, FindParameters),
}
