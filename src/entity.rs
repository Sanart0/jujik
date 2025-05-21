pub mod date;
pub mod kind;
pub mod owner;
pub mod permission;
pub mod size;

use crate::error::JujikError;
use date::EntityDate;
use kind::EntityKind;
use owner::EntityOwners;
use permission::EntityPermissions;
use serde::{Deserialize, Serialize};
use size::{EntitySize, EntitySizeKind};
use std::{
    fmt::Display,
    fs::{File, canonicalize, symlink_metadata},
    io::Read,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
};

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct Entity {
    global_path: PathBuf,
    name: String,
    extension: Option<String>,
    kind: EntityKind,
    permissions: EntityPermissions,
    owners: EntityOwners,
    size: EntitySize,
    date: EntityDate,
}

impl Entity {
    pub fn new(pathbuf: PathBuf) -> Result<Self, JujikError> {
        let path = pathbuf.as_path();

        Ok(Self {
            global_path: Self::get_global_path(path)?,
            name: Self::get_name(path)?,
            extension: Self::get_extension(path)?,
            kind: Self::get_kind(path)?,
            permissions: Self::get_permissions(path)?,
            owners: Self::get_owners(path)?,
            size: Self::get_size(path)?,
            date: Self::get_date(path)?,
        })
    }

    pub fn ghost(pathbuf: PathBuf, name: String, kind: EntityKind) -> Result<Self, JujikError> {
        let extension = pathbuf
            .extension()
            .and_then(|n| n.to_str())
            .and_then(|n| Some(n.to_string()));

        let permissions = EntityPermissions::new(match kind {
            EntityKind::File => 0o644,
            EntityKind::Directory => 0o755,
            _ => 0o000,
        });

        Ok(Self {
            global_path: pathbuf,
            name,
            extension,
            kind,
            permissions,
            owners: EntityOwners::current()?,
            size: EntitySize::default(),
            date: EntityDate::now(),
        })
    }
}

impl Entity {
    pub fn path(&self) -> PathBuf {
        self.global_path.clone()
    }

    pub fn path_str(&self) -> String {
        if let Some(path_str) = self.global_path.to_str().and_then(|p| Some(p.to_string())) {
            path_str
        } else {
            String::new()
        }
    }

    pub fn path_dir(&self) -> PathBuf {
        if let Some(path) = self.global_path.parent() {
            path.to_path_buf()
        } else {
            PathBuf::new()
        }
    }

    pub fn path_dir_str(&self) -> String {
        if let Some(path) = self
            .global_path
            .parent()
            .and_then(|p| p.to_str())
            .and_then(|p| Some(p.to_string()))
        {
            path
        } else {
            String::new()
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn name_with_extension(&self) -> String {
        if let Some(extension) = &self.extension {
            self.name.clone() + "." + extension
        } else {
            self.name.clone()
        }
    }

    pub fn extension(&self) -> &Option<String> {
        &self.extension
    }

    pub fn extension_str(&self) -> String {
        if let Some(extension) = &self.extension {
            extension.clone()
        } else {
            "None".to_string()
        }
    }

    pub fn kind(&self) -> &EntityKind {
        &self.kind
    }

    pub fn permissions(&self) -> &EntityPermissions {
        &self.permissions
    }

    pub fn owners(&self) -> &EntityOwners {
        &self.owners
    }

    pub fn size(&self) -> &EntitySize {
        &self.size
    }

    pub fn date(&self) -> &EntityDate {
        &self.date
    }

    pub fn exists(&self) -> bool {
        self.path().exists()
    }

    pub fn is_file(&self) -> bool {
        if let EntityKind::File = self.kind {
            true
        } else {
            false
        }
    }

    pub fn is_dir(&self) -> bool {
        if let EntityKind::Directory = self.kind {
            true
        } else {
            false
        }
    }

    pub fn content(&self) -> Result<String, JujikError> {
        let mut file = File::open(self.path())?;
        let mut content = String::new();

        file.read_to_string(&mut content)?;

        Ok(content)
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
        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
            Ok(name.to_string())
        } else if let Some(path_str) = path.to_str() {
            Ok(match path_str {
                "." => ".".to_string(),
                ".." => "..".to_string(),
                "/" => "/".to_string(),
                _ => "".to_string(),
            })
        } else {
            //TODO Handle error
            Err(JujikError::Other(format!("")))
        }
    }

    fn get_extension(path: &Path) -> Result<Option<String>, JujikError> {
        if let Some(extension) = path.extension().and_then(|n| n.to_str()) {
            Ok(Some(extension.to_string()))
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

    fn get_size(path: &Path) -> Result<EntitySize, JujikError> {
        let metadata = symlink_metadata(path)?;

        Ok(EntitySize::new(metadata.len()))
    }

    fn get_date(path: &Path) -> Result<EntityDate, JujikError> {
        let metadata = symlink_metadata(path)?;

        Ok(EntityDate::new(metadata.modified()?, metadata.created()?))
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity")
            .field("global_path", &self.global_path)
            .field("name", &self.name)
            .field("extension", &self.extension_str())
            .field("kind", &self.kind)
            .field("permissions", &self.permissions)
            .field("owners", &self.owners)
            .field("size", &self.size)
            .field("date", &self.date)
            .finish()
    }
}
