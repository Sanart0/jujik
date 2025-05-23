use crate::{
    config::Config,
    entity::{Entity, owner::EntityOwners, permission::EntityPermissions},
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
    Uptade,

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
    CreateTab(TabKind, PathBuf),
    DeleteTab(usize, Tab),
    ChangeTabName(usize, Tab, String),
    ChangeTabDirectory(usize, Tab, Option<PathBuf>),
    ChangeTabPosition(usize, usize, Tab),
    NewTab(Option<usize>, Tab),

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
}
