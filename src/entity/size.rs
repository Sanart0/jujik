use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum EntitySizeKind {
    #[default]
    None,
    Byte,
    KiloByte,
    MegaByte,
    GigaByte,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EntitySize {
    kind: EntitySizeKind,
    size_byte: u64,
    size: f32,
}

impl EntitySize {
    pub fn new(size_byte: u64) -> Self {
        let (kind, size) = Self::range_size(size_byte);

        Self {
            kind,
            size_byte,
            size,
        }
    }

    fn range_size(size_byte: u64) -> (EntitySizeKind, f32) {
        let gb = EntitySizeKind::GigaByte.value();
        let mb = EntitySizeKind::MegaByte.value();
        let kb = EntitySizeKind::KiloByte.value();

        if size_byte > gb {
            (EntitySizeKind::GigaByte, size_byte as f32 / gb as f32)
        } else if size_byte > mb {
            (EntitySizeKind::MegaByte, size_byte as f32 / mb as f32)
        } else if size_byte > kb {
            (EntitySizeKind::KiloByte, size_byte as f32 / kb as f32)
        } else {
            (EntitySizeKind::Byte, size_byte as f32)
        }
    }

    pub fn add(&mut self, size_byte: u64) {
        self.size_byte += size_byte;

        let (kind, size) = Self::range_size(self.size_byte);

        self.kind = kind;
        self.size = size;
    }
}

impl Eq for EntitySize {}

impl PartialEq for EntitySize {
    fn eq(&self, other: &Self) -> bool {
        self.size.to_bits() == other.size.to_bits()
    }
}

impl Hash for EntitySize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.size.to_bits().hash(state);
    }
}

impl EntitySizeKind {
    fn value(&self) -> u64 {
        match self {
            EntitySizeKind::None => 0,
            EntitySizeKind::Byte => 1,
            EntitySizeKind::KiloByte => 1024,
            EntitySizeKind::MegaByte => 1024 * 1024,
            EntitySizeKind::GigaByte => 1024 * 1024 * 1024,
        }
    }
}

impl PartialOrd for EntitySize {
    fn lt(&self, other: &Self) -> bool {
        self.size_byte < other.size_byte
    }

    fn le(&self, other: &Self) -> bool {
        self.size_byte <= other.size_byte
    }

    fn gt(&self, other: &Self) -> bool {
        self.size_byte > other.size_byte
    }

    fn ge(&self, other: &Self) -> bool {
        self.size_byte >= other.size_byte
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.size_byte.partial_cmp(&other.size_byte)
    }
}

impl Ord for EntitySize {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size_byte.cmp(&other.size_byte)
    }
}

impl Display for EntitySizeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EntitySizeKind::None => "None",
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
        if let EntitySizeKind::None = self.kind {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{:.2} {}", self.size, self.kind)
        }
    }
}

impl Debug for EntitySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
