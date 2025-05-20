use crate::{entity::Entity, error::JujikError};
use std::{fs::read_dir, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TabKind {
    None,
    Entitys,
    View,
    Editor,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TabContent {
    None,
    Entitys(Vec<Entity>),
    View(PathBuf),
    Editor(PathBuf),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tab {
    name: String,
    pathbuf: PathBuf,
    content: TabContent,
}

impl Tab {
    pub fn new(tab_kind: TabKind, pathbuf: PathBuf) -> Result<Self, JujikError> {
        Ok(Self {
            name: Entity::get_name(pathbuf.as_path())?,
            pathbuf: pathbuf.clone(),
            content: match tab_kind {
                TabKind::Entitys => TabContent::Entitys(Tab::read_dir(pathbuf)?),
                TabKind::View => TabContent::View(pathbuf),
                TabKind::Editor => TabContent::Editor(pathbuf),
                TabKind::None => TabContent::None,
            },
        })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.pathbuf.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name.clone_from(&name);
    }

    pub fn path_str(&self) -> String {
        if let Some(path_str) = self.pathbuf.to_str() {
            path_str.to_string()
        } else {
            String::new()
        }
    }

    pub fn content(&self) -> &TabContent {
        &self.content
    }

    pub fn entitys(&self) -> Option<Vec<Entity>> {
        if let TabContent::Entitys(entitys) = &self.content {
            Some(entitys.clone())
        } else {
            None
        }
    }

    pub fn entitys_mut(&mut self) -> Option<&mut Vec<Entity>> {
        if let TabContent::Entitys(entitys) = &mut self.content {
            Some(entitys)
        } else {
            None
        }
    }

    fn read_dir(pathbuf: PathBuf) -> Result<Vec<Entity>, JujikError> {
        let mut entitys: Vec<Entity> = Vec::new();

        for dir_entry in read_dir(pathbuf.clone())? {
            if let Ok(entity) = Entity::new(dir_entry?.path()) {
                entitys.push(entity);
            } else {
                //TODO Handle error
            }
        }

        Ok(entitys)
    }

    pub fn change_dir(&mut self, pathbuf: PathBuf) -> Result<(), JujikError> {
        if let TabContent::Entitys(_) = &self.content {
            *self = Tab::new(TabKind::Entitys, pathbuf)?;
        }

        Ok(())
    }

    pub fn change_dir_back(&mut self) -> Result<(), JujikError> {
        match &self.content {
            TabContent::Entitys(_) => {
                if let Some(parent) = self.path().parent() {
                    *self = Tab::new(TabKind::Entitys, parent.to_path_buf())?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn update_entitys(&mut self) -> Result<(), JujikError> {
        let pathbuf = self.pathbuf.clone();

        if let Some(entitys) = self.entitys_mut() {
            *entitys = Tab::read_dir(pathbuf)?;
        }

        Ok(())
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            name: String::new(),
            pathbuf: PathBuf::new(),
            content: TabContent::None,
        }
    }
}
