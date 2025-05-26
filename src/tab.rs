use crate::{
    entity::{
        Entity,
        find::{EntitysFinder, FindParameters},
    },
    error::JujikError,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::read_dir, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TabKind {
    None,
    Entitys,
    View,
    Editor,
    Find,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    Name,
    Extension,
    Permissions,
    Owners,
    Size,
    Modification,
    Creation,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SortBy {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TabContent {
    #[default]
    None,
    Entitys(SortBy, PathBuf, Vec<Entity>),
    View(Entity),
    Editor(Entity),
    Find(EntitysFinder),
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Tab {
    name: String,
    content: TabContent,
}

impl Tab {
    pub fn tab_entitys(pathbuf: PathBuf) -> Result<Self, JujikError> {
        Ok(Self {
            name: format!(
                "{}: {}",
                TabKind::Entitys,
                Entity::get_name(pathbuf.as_path())?
            ),
            content: TabContent::Entitys(
                SortBy::default(),
                pathbuf.clone(),
                Tab::read_dir(pathbuf.clone())?,
            ),
        })
    }

    pub fn tab_view(pathbuf: PathBuf) -> Result<Self, JujikError> {
        Ok(Self {
            name: format!(
                "{}: {}",
                TabKind::View,
                Entity::get_name(pathbuf.as_path())?
            ),
            content: TabContent::View(Entity::new(pathbuf.clone())?),
        })
    }

    pub fn tab_editor(pathbuf: PathBuf) -> Result<Self, JujikError> {
        Ok(Self {
            name: format!(
                "{}: {}",
                TabKind::Editor,
                Entity::get_name(pathbuf.as_path())?
            ),
            content: TabContent::Editor(Entity::new(pathbuf.clone())?),
        })
    }

    pub fn tab_finder(parameters: FindParameters) -> Result<Self, JujikError> {
        Ok(Self {
            name: format!(
                "{}: {}",
                TabKind::Find,
                Entity::get_name(parameters.path.as_path())?
            ),
            content: TabContent::Find(EntitysFinder::find(parameters)?),
        })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(&self) -> PathBuf {
        match &self.content {
            TabContent::Entitys(_, pathbuf, _) => pathbuf.clone(),
            TabContent::View(entity) => entity.path_dir(),
            TabContent::Editor(entity) => entity.path_dir(),
            TabContent::Find(finder) => finder.parameters().path,
            TabContent::None => PathBuf::new(),
        }
    }

    pub fn sortby(&self) -> SortBy {
        if let TabContent::Entitys(sortby, _, _) = &self.content {
            sortby.clone()
        } else {
            SortBy::default()
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name.clone_from(&name);
    }

    pub fn set_sortby(&mut self, new_sortby: &SortBy) {
        if let TabContent::Entitys(sortby, _, _) = &mut self.content {
            sortby.clone_from(new_sortby);
        }
    }

    pub fn path_str(&self) -> String {
        if let Some(path_str) = self.path().to_str() {
            path_str.to_string()
        } else {
            String::new()
        }
    }

    pub fn content(&self) -> &TabContent {
        &self.content
    }

    pub fn entitys(&self) -> Option<Vec<Entity>> {
        if let TabContent::Entitys(_, _, entitys) = &self.content {
            Some(entitys.clone())
        } else {
            None
        }
    }

    pub fn entitys_mut(&mut self) -> Option<&mut Vec<Entity>> {
        if let TabContent::Entitys(_, _, entitys) = &mut self.content {
            Some(entitys)
        } else {
            None
        }
    }

    pub fn sort(&mut self) {
        if let TabContent::Entitys(sortby, _, entitys) = &mut self.content {
            match sortby.field {
                SortField::Name => entitys.sort_by(|e1, e2| e1.name().cmp(&e2.name())),
                SortField::Extension => {
                    entitys.sort_by(|e1, e2| e1.extension().cmp(e2.extension()))
                }
                SortField::Permissions => {
                    entitys.sort_by(|e1, e2| e1.permissions().cmp(e2.permissions()))
                }
                SortField::Owners => entitys.sort_by(|e1, e2| e1.owners().cmp(e2.owners())),
                SortField::Size => entitys.sort_by(|e1, e2| e1.size().cmp(e2.size())),
                SortField::Modification => {
                    entitys.sort_by(|e1, e2| e1.modification().cmp(e2.modification()))
                }
                SortField::Creation => entitys.sort_by(|e1, e2| e1.creation().cmp(e2.creation())),
            }

            match sortby.direction {
                SortDirection::Ascending => {}
                SortDirection::Descending => entitys.reverse(),
            }
        }
    }

    fn read_dir(pathbuf: PathBuf) -> Result<Vec<Entity>, JujikError> {
        let mut entitys: Vec<Entity> = Vec::new();

        for dir_entry in read_dir(pathbuf.clone())? {
            if let Ok(entity) = Entity::new(dir_entry?.path()) {
                entitys.push(entity);
            }
        }

        Ok(entitys)
    }

    pub fn change_dir(&mut self, pathbuf: PathBuf) -> Result<(), JujikError> {
        if let TabContent::Entitys(_, _, _) = &self.content {
            *self = Tab::tab_entitys(pathbuf)?;
        }

        Ok(())
    }

    pub fn change_dir_back(&mut self) -> Result<(), JujikError> {
        match &self.content {
            TabContent::Entitys(_, _, _) => {
                if let Some(parent) = self.path().parent() {
                    *self = Tab::tab_entitys(parent.to_path_buf())?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn clear_entitys(&mut self) {
        if let Some(entitys) = self.entitys_mut() {
            entitys.clear();
        }
    }

    pub fn update_entitys(&mut self) -> Result<(), JujikError> {
        if let TabContent::Entitys(_, pathbuf, entitys) = &mut self.content {
            *entitys = Tab::read_dir(pathbuf.clone())?;
        }

        Ok(())
    }
}

impl Display for TabKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TabKind::Entitys => "Entitys",
                TabKind::View => "View",
                TabKind::Editor => "Editor",
                TabKind::Find => "Find",
                TabKind::None => "None",
            }
        )
    }
}
