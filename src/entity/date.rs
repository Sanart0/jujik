use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::SystemTime};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
pub struct EntityDate {
    date: DateTime<Utc>,
}

impl EntityDate {
    pub fn new(date: SystemTime) -> Self {
        Self {
            date: EntityDate::system_time_to_datetime(date),
        }
    }

    pub fn now() -> Self {
        Self { date: Utc::now() }
    }

    fn system_time_to_datetime(t: SystemTime) -> DateTime<Utc> {
        if let Some(date_time) = t.duration_since(SystemTime::UNIX_EPOCH).ok().and_then(|d| {
            Utc.timestamp_opt(d.as_secs() as i64, d.subsec_nanos())
                .single()
        }) {
            date_time
        } else {
            DateTime::default()
        }
    }

    pub fn date_str(&self) -> String {
        self.date.format("%d/%m/%Y").to_string()
    }
}

impl Debug for EntityDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityDate")
            .field("date", &self.date_str())
            .finish()
    }
}
