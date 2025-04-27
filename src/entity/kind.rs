use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum EntityKind {
    File,
    Directory,
    Symlink,
    Block,
    Character,
    Pipe,
    Socket,
    Unknown,
}

impl Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EntityKind::File => "-",
                EntityKind::Directory => "d",
                EntityKind::Symlink => "l",
                EntityKind::Block => "b",
                EntityKind::Character => "c",
                EntityKind::Pipe => "p",
                EntityKind::Socket => "s",
                EntityKind::Unknown => "?",
            }
        )
    }
}
