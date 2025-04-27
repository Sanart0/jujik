pub mod kind;
pub mod owner;
pub mod permission;

use crate::error::JujikError;
use kind::EntityKind;
use owner::EntityOwners;
use permission::EntityPermissions;
use std::{
    fmt::Display,
    fs::{canonicalize, symlink_metadata},
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Entity {
    global_path: PathBuf,
    name: String,
    extension: Option<String>,
    kind: EntityKind,
    permissions: EntityPermissions,
    owners: EntityOwners,
}

impl Entity {
    pub fn new(path_buf: PathBuf) -> Result<Entity, JujikError> {
        let path = path_buf.as_path();

        let global_path = Self::get_global_path(path)?;
        let name = Self::get_name(path)?;
        let extension = Self::get_extansion(path)?;
        let kind = Self::get_kind(path)?;
        let permissions = Self::get_permissions(path)?;
        let owners = Self::get_owners(path)?;

        Ok(Self {
            global_path,
            name,
            extension,
            kind,
            permissions,
            owners,
        })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Entity {
    fn get_global_path(path: &Path) -> Result<PathBuf, JujikError> {
        match canonicalize(path) {
            Ok(global_path) => Ok(global_path),
            Err(err) => Err(JujikError::from(err)),
        }
    }

    pub fn get_name(path: &Path) -> Result<String, JujikError> {
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            let mut name_split = name.split('.').collect::<Vec<&str>>();
            if name_split.len() > 1 {
                name_split.pop();
                Ok(name_split.join("."))
            } else {
                Ok(name.to_string())
            }
        } else if let Some(path_str) = path.to_str() {
            Ok(match path_str {
                "." => ".".to_string(),
                ".." => "..".to_string(),
                "/" => "/".to_string(),
                _ => "".to_string(),
            })
        } else {
            //TODO Handle error
            Err(JujikError::None)
        }
    }

    fn get_extansion(path: &Path) -> Result<Option<String>, JujikError> {
        if let Some(extansion) = path.extension() {
            Ok(extansion.to_str().map(|extansion| extansion.to_string()))
        } else {
            //TODO Handle error
            Ok(None)
        }
    }

    fn get_kind(path: &Path) -> Result<EntityKind, JujikError> {
        let file_type = symlink_metadata(path)?.file_type();

        Ok({
            if file_type.is_file() {
                EntityKind::File
            } else if file_type.is_dir() {
                EntityKind::Directory
            } else if file_type.is_symlink() {
                EntityKind::Symlink
            } else if file_type.is_block_device() {
                EntityKind::Block
            } else if file_type.is_char_device() {
                EntityKind::Character
            } else if file_type.is_fifo() {
                EntityKind::Pipe
            } else if file_type.is_socket() {
                EntityKind::Socket
            } else {
                EntityKind::Unknown
            }
        })
    }

    fn get_permissions(path: &Path) -> Result<EntityPermissions, JujikError> {
        let mode = symlink_metadata(path)?.mode();

        Ok(EntityPermissions::new(mode))
    }

    fn get_owners(path: &Path) -> Result<EntityOwners, JujikError> {
        let metadata = symlink_metadata(path)?;
        let (uid, gid) = (metadata.uid(), metadata.gid());

        EntityOwners::new(uid, gid)
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let extansion = if let Some(extension) = &self.extension {
            extension
        } else {
            ""
        };
        f.debug_struct("Entity")
            .field("global_path", &self.global_path)
            .field("name", &self.name)
            .field("extension", &extansion)
            .field("kind", &self.kind)
            .field("permissions", &self.permissions)
            .field("owners", &self.owners)
            .finish()
    }
}
