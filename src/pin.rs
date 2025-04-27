use crate::{entity::Entity, error::JujikError};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    name: String,
    path: PathBuf,
}

impl Pin {
    pub fn new(pathbuf: PathBuf) -> Result<Self, JujikError> {
        Ok(Self {
            name: Entity::get_name(pathbuf.as_path())?,
            path: pathbuf,
        })
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Default for Pin {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
        }
    }
}
