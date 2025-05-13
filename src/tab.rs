use crate::{entity::Entity, error::JujikError};
use std::{fs::read_dir, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TabKind {
    None,
    Entitys,
    Editor,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TabContent {
    None,
    Entitys(Vec<Entity>),
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
                TabKind::Editor => TabContent::Editor(pathbuf),
                TabKind::None => TabContent::None,
            },
        })
    }

    pub fn empty() -> Self {
        Self {
            name: String::new(),
            pathbuf: PathBuf::new(),
            content: TabContent::None,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn pathbuf(&self) -> PathBuf {
        self.pathbuf.clone()
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
        match &self.content {
            TabContent::Entitys(entitys) => Some(entitys.clone()),
            _ => None,
        }
    }

    pub fn entitys_mut(&mut self) -> Option<&mut Vec<Entity>> {
        match &mut self.content {
            TabContent::Entitys(entitys) => Some(entitys),
            _ => None,
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
        match &self.content {
            TabContent::Entitys(_) => {
                *self = Tab::new(TabKind::Entitys, pathbuf)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn change_dir_back(&mut self) -> Result<(), JujikError> {
        match &self.content {
            TabContent::Entitys(_) => {
                if let Some(parent) = self.pathbuf().parent() {
                    *self = Tab::new(TabKind::Entitys, parent.to_path_buf())?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
