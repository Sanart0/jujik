use crate::{entity::Entity, error::JujikError};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    name: String,
    pathbuf: PathBuf,
}

impl Pin {
    pub fn new(pathbuf: PathBuf) -> Result<Self, JujikError> {
        Ok(Self {
            name: Entity::get_name(pathbuf.as_path())?,
            pathbuf,
        })
    }

    pub fn empty() -> Self {
        Self {
            name: String::new(),
            pathbuf: PathBuf::new(),
        }
    }

    pub fn from(name: String, pathbuf: PathBuf) -> Self {
        Self { name, pathbuf }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.pathbuf.clone()
    }

    pub fn path_str(&self) -> String {
        if let Some(path_str) = self.pathbuf.to_str() {
            path_str.to_string()
        } else {
            String::new()
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name.clone_from(&name);
    }

    pub fn set_path(&mut self, pathbuf: PathBuf) {
        self.pathbuf.clone_from(&pathbuf);
    }
}

impl Default for Pin {
    fn default() -> Self {
        Self {
            name: String::new(),
            pathbuf: PathBuf::new(),
        }
    }
}
