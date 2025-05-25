use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    fs::Permissions,
    os::unix::fs::PermissionsExt,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EntityPermissionsKind {
    Read,
    Write,
    Execute,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EntityPermissionsCategory {
    User,
    Group,
    Other,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
pub struct EntityPermissions {
    mode: u32,
}

impl EntityPermissions {
    pub fn new(mode: u32) -> Self {
        Self { mode: mode & 0o777 }
    }

    pub fn has(
        &self,
        categoty: EntityPermissionsCategory,
        permission: EntityPermissionsKind,
    ) -> bool {
        let c = categoty.mask();
        let s = categoty.shift();
        let p = permission.mask();

        p & ((self.mode & c) >> s) != 0
    }

    pub fn set(&mut self, categoty: EntityPermissionsCategory, permission: EntityPermissionsKind) {
        let s = categoty.shift();
        let p = permission.mask();

        self.mode |= p << s;
    }

    pub fn unset(
        &mut self,
        categoty: EntityPermissionsCategory,
        permission: EntityPermissionsKind,
    ) {
        let s = categoty.shift();
        let p = permission.mask();

        self.mode &= !(p << s);
    }

    pub fn symbolic(&self) -> String {
        let u = EntityPermissionsCategory::User;
        let g = EntityPermissionsCategory::Group;
        let o = EntityPermissionsCategory::Other;
        let r = EntityPermissionsKind::Read;
        let w = EntityPermissionsKind::Write;
        let e = EntityPermissionsKind::Execute;

        let ur = if self.has(u, r) { "r" } else { "-" };
        let uw = if self.has(u, w) { "w" } else { "-" };
        let ue = if self.has(u, e) { "e" } else { "-" };
        let gr = if self.has(g, r) { "r" } else { "-" };
        let gw = if self.has(g, w) { "w" } else { "-" };
        let ge = if self.has(g, e) { "e" } else { "-" };
        let or = if self.has(o, r) { "r" } else { "-" };
        let ow = if self.has(o, w) { "w" } else { "-" };
        let oe = if self.has(o, e) { "e" } else { "-" };

        format!("{}{}{}{}{}{}{}{}{}", ur, uw, ue, gr, gw, ge, or, ow, oe)
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }
}

impl EntityPermissionsCategory {
    pub fn mask(&self) -> u32 {
        match self {
            EntityPermissionsCategory::User => 0o700,
            EntityPermissionsCategory::Group => 0o070,
            EntityPermissionsCategory::Other => 0o007,
        }
    }

    pub fn shift(&self) -> u32 {
        match self {
            EntityPermissionsCategory::User => 0o6,
            EntityPermissionsCategory::Group => 0o3,
            EntityPermissionsCategory::Other => 0o0,
        }
    }
}

impl EntityPermissionsKind {
    pub fn mask(&self) -> u32 {
        match self {
            EntityPermissionsKind::Read => 0o4,
            EntityPermissionsKind::Write => 0o2,
            EntityPermissionsKind::Execute => 0o1,
        }
    }
}

impl Into<Permissions> for EntityPermissions {
    fn into(self) -> Permissions {
        Permissions::from_mode(self.mode)
    }
}

impl Display for EntityPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:o})", self.symbolic(), self.mode)
    }
}

impl Debug for EntityPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
