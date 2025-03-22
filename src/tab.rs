use std::path::PathBuf;

use crate::entity::Entity;

pub struct Tab {
    name: String,
    path: PathBuf,
    entity: Vec<Entity>
}
