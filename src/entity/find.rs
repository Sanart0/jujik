use std::{
    collections::VecDeque,
    fs::{read_dir, symlink_metadata},
    path::PathBuf,
};

use crate::error::JujikError;

use super::{
    Entity, date::EntityDate, kind::EntityKind, owner::EntityOwners, permission::EntityPermissions,
    size::EntitySize,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FindParameters {
    pub regex: String,
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub kind: EntityKind,
    pub permissions: EntityPermissions,
    pub owners: EntityOwners,
    pub size: (EntitySize, EntitySize),
    pub date: (EntityDate, EntityDate),
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct EntitysFinder {
    parameters: FindParameters,
    entitys: Vec<Entity>,
}

impl EntitysFinder {
    pub fn find(parameters: FindParameters) -> Result<Self, JujikError> {
        let mut entitys: Vec<Entity> = Vec::new();
        let mut read_dirs: VecDeque<PathBuf> = VecDeque::new();

        entitys.push(Entity::new(parameters.path.clone())?);
        read_dirs.push_back(parameters.path.clone());

        while let Some(pathbuf) = read_dirs.pop_front() {
            if let Ok(metadata) = symlink_metadata(pathbuf.as_path()) {
                if metadata.is_dir() {
                    if let Ok(entity) = Entity::new(pathbuf.clone()) {
                        entitys.push(entity);
                    }
                    if let Ok(read_dir) = read_dir(pathbuf.as_path()) {
                        for dir_entry in read_dir {
                            if let Ok(dir_entry) = dir_entry {
                                read_dirs.push_back(dir_entry.path());
                            }
                        }
                    }
                } else if metadata.is_file() {
                    if let Ok(entity) = Entity::new(pathbuf) {
                        entitys.push(entity);
                    }
                }
            }
        }

        Ok(Self {
            parameters,
            entitys,
        })
    }

    pub fn parameters(&self) -> FindParameters {
        self.parameters.clone()
    }

    pub fn entitys(&self) -> Vec<Entity> {
        self.entitys.clone()
    }
}

impl Default for FindParameters {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/"),
            regex: String::new(),
            name: String::new(),
            extension: String::new(),
            kind: EntityKind::default(),
            permissions: EntityPermissions::default(),
            owners: EntityOwners::default(),
            size: (EntitySize::default(), EntitySize::default()),
            date: (EntityDate::default(), EntityDate::default()),
        }
    }
}
