use std::{fs::read_dir, path::PathBuf};

use crate::{entity::Entity, error::JujikError};

#[derive(Debug, PartialEq, Eq)]
pub struct Tab {
    name: String,
    entitys: Vec<Entity>,
}

impl Tab {
    pub fn new(pathbuf: PathBuf) -> Result<Self, JujikError> {
        let mut entitys: Vec<Entity> = Vec::new();

        for dir_entry in read_dir(pathbuf.clone())? {
            if let Ok(entity) = Entity::new(dir_entry?.path()) {
                entitys.push(entity);
            } else {
                //TODO Handle error
            }
        }
        Ok(Self {
            name: Entity::get_name(pathbuf.as_path())?,
            entitys,
        })
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_entitys(&self) -> Vec<Entity> {
        self.entitys.clone()
    }
}
