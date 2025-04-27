use std::fmt::{Debug, Display};

use crate::error::JujikError;
use nix::unistd::{Gid, Group, Uid, User};

#[derive(PartialEq, Eq, Default, Clone)]
pub struct EntityOwners {
    uid: u32,
    gid: u32,
    username: String,
    groupname: String,
}

impl EntityOwners {
    pub fn new(uid: u32, gid: u32) -> Result<Self, JujikError> {
        let username = {
            if let Some(user) = User::from_uid(Uid::from_raw(uid))? {
                user.name
            } else {
                String::new()
            }
        };
        let groupname = {
            if let Some(group) = Group::from_gid(Gid::from_raw(gid))? {
                group.name
            } else {
                String::new()
            }
        };

        Ok(Self {
            uid,
            gid,
            username,
            groupname,
        })
    }
}

impl Display for EntityOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for EntityOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}):{}({})",
            self.username, self.uid, self.groupname, self.gid
        )
    }
}
