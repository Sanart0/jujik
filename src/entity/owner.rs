use std::fmt::{Debug, Display};

use crate::error::JujikError;
use nix::unistd::{Gid, Group, Uid, User, getgid, getuid};
use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
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

    pub fn current() -> Result<Self, JujikError> {
        if let Some(user) = User::from_uid(getuid())? {
            if let Some(group) = Group::from_gid(getgid())? {
                return Ok(Self {
                    uid: user.uid.as_raw(),
                    gid: user.gid.as_raw(),
                    username: user.name,
                    groupname: group.name,
                });
            }
        }

        Ok(Self::default())
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn gid(&self) -> u32 {
        self.gid
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn groupname(&self) -> String {
        self.groupname.clone()
    }

    pub fn set_username(&mut self, username: String) -> Result<(), JujikError> {
        if let Some(user) = User::from_name(username.as_str())? {
            self.username.clone_from(&username);
            self.uid = user.uid.as_raw();
        } else {
            return Err(JujikError::Other(format!(
                "Can not set User from username: {}",
                username
            )));
        }

        Ok(())
    }

    pub fn set_groupname(&mut self, groupname: String) -> Result<(), JujikError> {
        if let Some(group) = Group::from_name(groupname.as_str())? {
            self.groupname.clone_from(&groupname);
            self.gid = group.gid.as_raw();
        } else {
            return Err(JujikError::Other(format!(
                "Can not set Group from groupname: {}",
                groupname
            )));
        }

        Ok(())
    }
}

impl Display for EntityOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}):{}({})",
            self.username, self.uid, self.groupname, self.gid
        )
    }
}

impl Debug for EntityOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
