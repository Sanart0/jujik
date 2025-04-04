use std::path::PathBuf;

#[derive(Debug)]
pub struct Pin {
    name: String,
    path: PathBuf,
}

impl Default for Pin {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
        }
    }
}
