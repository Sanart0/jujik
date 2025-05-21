use std::{fmt::Debug, time::SystemTime};

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct EntityDate {
    modification: Option<DateTime<Utc>>,
    creation: Option<DateTime<Utc>>,
}

impl EntityDate {
    pub fn new(modification: SystemTime, creation: SystemTime) -> Self {
        Self {
            modification: EntityDate::system_time_to_datetime(modification),
            creation: EntityDate::system_time_to_datetime(creation),
        }
    }

    pub fn now() -> Self {
        Self {
            modification: Some(Utc::now()),
            creation: Some(Utc::now()),
        }
    }

    fn system_time_to_datetime(t: SystemTime) -> Option<DateTime<Utc>> {
        t.duration_since(SystemTime::UNIX_EPOCH).ok().and_then(|d| {
            Utc.timestamp_opt(d.as_secs() as i64, d.subsec_nanos())
                .single()
        })
    }

    pub fn modification_str(&self) -> String {
        if let Some(modification) = self.modification {
            modification.format("%d/%m/%Y").to_string()
        } else {
            String::new()
        }
    }

    pub fn creation_str(&self) -> String {
        if let Some(creation) = self.creation {
            creation.format("%d/%m/%Y").to_string()
        } else {
            String::new()
        }
    }
}

impl Debug for EntityDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityDate")
            .field("modification", &self.modification_str())
            .field("creation", &self.creation_str())
            .finish()
    }
}
