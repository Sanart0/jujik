use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum EntitySizeKind {
    Byte,
    KiloByte,
    MegaByte,
    GigaByte,
}

#[derive(Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct EntitySize {
    size: u64,
}

impl EntitySize {
    pub fn new(size: u64) -> Self {
        Self { size }
    }

    pub fn value(&self) -> (u64, EntitySizeKind) {
        let gb = EntitySizeKind::GigaByte.value();
        let mb = EntitySizeKind::GigaByte.value();
        let kb = EntitySizeKind::GigaByte.value();

        if self.size > gb {
            (self.size / gb, EntitySizeKind::GigaByte)
        } else if self.size > mb {
            (self.size / mb, EntitySizeKind::MegaByte)
        } else if self.size > kb {
            (self.size / kb, EntitySizeKind::KiloByte)
        } else {
            (self.size, EntitySizeKind::Byte)
        }
    }
}

impl EntitySizeKind {
    fn value(&self) -> u64 {
        match self {
            EntitySizeKind::Byte => 1,
            EntitySizeKind::KiloByte => 1024,
            EntitySizeKind::MegaByte => 1024 * 1024,
            EntitySizeKind::GigaByte => 1024 * 1024 * 1024,
        }
    }
}

impl Display for EntitySizeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EntitySizeKind::Byte => "B",
                EntitySizeKind::KiloByte => "KB",
                EntitySizeKind::MegaByte => "MB",
                EntitySizeKind::GigaByte => "GB",
            }
        )
    }
}

impl Display for EntitySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (size,  kind) = self.value();
        write!(f, "{} {}", size, kind)
    }
}

impl Debug for EntitySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
