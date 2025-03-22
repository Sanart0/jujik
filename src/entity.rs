use std::path::{Path, PathBuf};

pub struct Entity {
    global_path: String,
    name: String,
    extansion: Option<String>,
    kind: u8,
    rules: (u8, u8, u8),
    entitys: Vec<Entity>,
}

impl Entity {
    pub fn new(path_buf: PathBuf) -> Entity {
        let path = path_buf.as_path();

        Self {
            global_path: Self::get_global_path(path),
            name: Self::get_name(path),
            extansion: Self::get_extansion(path),
            kind: todo!(),
            rules: todo!(),
            entitys: todo!(),
        }

    }

    fn get_global_path(path: &Path) -> String {
        if let Some(parent) = path.parent() {
            if let Some(parent) = parent.to_str() {
                parent.to_string()
            } else {
                //TODO Handle error
                String::new()
            }
        } else {
            //TODO Handle error
            String::new()
        }
    }

    fn get_name(path: &Path) -> String {
        if let Some(name) = path.file_name() {
            if let Some(name) = name.to_str() {
                name.to_string()
            } else {
                //TODO Handle error
                String::new()
            }
        } else {
            //TODO Handle error
            String::new()
        }
    }

    fn get_extansion(path: &Path) -> Option<String> {
        if let Some(extansion) = path.extension() {
            extansion.to_str().map(|extansion| extansion.to_string())
        } else {
            //TODO Handle error
            None
        }
    }
}
